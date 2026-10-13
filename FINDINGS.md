# macOS 虚拟手柄 / PS4·PS5 串流输入 调研记录

本文记录本次调研与实测得到的结论，便于以后复用。所有"实测"都是在 macOS 26.5.1 (arm64, Apple M5) 上做的。

## 1. 为什么"用代码在 macOS 上造虚拟手柄"这条路走不通

### 1.1 Apple 的硬性要求

`IOHIDUserDeviceCreateWithProperties` 的官方头文件注释里写得很直白：

> In order to create the device, the entitlement `com.apple.developer.hid.virtual.device` is required to validate the source of the device.

这是一个 **restricted entitlement**，必须由 Apple 批准的 provisioning profile 背书，无法自行签发。

### 1.2 实测结果（`tools/iohid_probe.c`）

| 情况 | 结果 |
| --- | --- |
| 未签名、无 entitlement | `IOHIDUserDeviceCreateWithProperties` 返回 NULL，创建设备被拒 |
| ad-hoc 签名 + 把 entitlement 写进签名 | 进程启动即 **SIGKILL (exit 137)**，被 AMFI 杀掉 |

结论：所谓"改源码 + 自签名绕过"在 macOS 15+/26 上不可行。

### 1.3 macOS 15+ 的变化

`IOHIDUserDevice` 在 macOS 15 起被标记为 obsoleted，替代品是 CoreHID 的 `HIDVirtualDevice`，**需要同一个 entitlement**。换 API 不解决问题。

### 1.4 旁证

开源的 `xsyetopz/OpenJoystickDriver`（macOS 虚拟手柄工具）自己的 `Host.entitlements` 里就有：

```xml
<key>com.apple.developer.hid.virtual.device</key><true/>
<key>com.apple.developer.system-extension.install</key><true/>
<key>com.apple.developer.driverkit.userclient-access</key>...
```

也就是说：**自己编译它也拿不到能力，只能用官方预先签好的构建**。

## 2. libvirtualhid 的现状（本机实测）

`LizardByte/libvirtualhid` 的跨平台库是 MIT；但 Windows/macOS 的 broker、驱动、以及"授权/评估"相关源码是 **LB-SAL 1.0**（source-available）。

macOS 上创建虚拟手柄需要同时满足：

1. 安装已签名的 **Virtual HID Broker**（LaunchDaemon）；
2. broker 带 Apple 审批的 `com.apple.developer.hid.virtual.device`；
3. 机器已激活付费 license。

三者的缺省错误信息是递进的，很好区分：

| 状态 | 错误信息 |
| --- | --- |
| 没装 broker | `backend_unavailable: macOS broker directory is missing or insecure` |
| 装了 broker、没 license | `backend_unavailable: An active license is required` |
| 有 license、缺系统权限 | `backend_failure: Virtual HID creation failed; check broker signing and macOS Accessibility permission` |

排查命令：

```sh
ls -ld /Applications/VirtualHIDBroker.app
ls -ld /var/run/libvirtualhid
launchctl print system/dev.lizardbyte.app.libvirtualhid | head -6
/usr/local/bin/libvirtualhid-license status
```

注意：`/var/run/libvirtualhid` 目录属主必须是 root 且 group/other 不可写，socket 属主必须是 root——客户端会校验这两点，所以**无法用普通用户进程假冒**。

## 3. "用代码发按键"的可行路线对比

| 方案 | 需要硬件 | 花钱 | 要写代码 | 最小化仍生效 |
| --- | --- | --- | --- | --- |
| 合成键盘/鼠标事件（CoreGraphics） | 否 | 否 | 少许 | ❌ 需窗口焦点 |
| 虚拟手柄（libvirtualhid + license） | 否 | 是 | 否 | ✅ |
| 真硬件手柄桥接（RP2040 伪装 DS4） | 是（约 ¥25） | 否 | 固件 | ✅ |
| Windows VM + ViGEmBus | 否 | Windows 授权 | 少许 | ✅（游戏要跑在 VM 里） |
| 改串流客户端加本地输入源 | 否 | 否 | 中等 | ✅ |
| 自写 libchiaki 客户端 | 否 | 否 | 大 | ✅ |

关键区别是**输入走不走窗口系统**：键盘事件必须到达有焦点的窗口；手柄事件（SDL/IOKit HID 回调）与焦点无关。

## 4. DualShock 4 报文细节（可复用）

取自 libvirtualhid 的 MIT 部分，实测打包结果：

| 项目 | USB | 蓝牙 |
| --- | --- | --- |
| 输入报文长度 | 64 B | 78 B |
| Report ID | `0x01` | `0x11` |
| 输出报文长度 | 32 B（Report ID `0x05`） | 78 B |
| VID / PID | `0x054C` / `0x05C4` | 同左 |
| HID 描述符长度 | 467 B | 另一份 |

USB 输入报文布局（0 基）：

```
[0]      report id (0x01)
[1..4]   左摇杆 XY、右摇杆 XY（0x80 居中）
[5]      十字键 + 面键位
[6]      L1 R1 L2 R2 share options L3 R3
[7]      PS + touchpad 标志
[8]      L2 模拟量
[9]      R2 模拟量
[10..11] 传感器时间戳（随进程单调时钟变化）
[12..13] 陀螺仪 X ...
```

**注意 `[10..11]` 是时间戳**：同一状态在不同时刻打包，这 2 个字节会不同，属于正常现象（真手柄也这样）。

按键位（libvirtualhid 的逻辑按键，21 个）：

```
0 a/✕   1 b/○   2 x/□   3 y/△   4 back/share   5 start/options
6 guide/PS  7 L3  8 R3  9 L1  10 R1
11 dpad_up 12 dpad_down 13 dpad_left 14 dpad_right
15 misc1 16 touchpad 17..20 paddle1..4
```

## 5. 硬件选型（想做"真手柄"伪装时）

| 芯片 | 任意 USB 设备类 | 蓝牙经典 (BR/EDR) | BLE |
| --- | --- | --- | --- |
| ESP32（原版） | ✗ | ✓ | ✓ |
| ESP32-S2 | ✓ | ✗ | ✗ |
| ESP32-S3 | ✓ | ✗ | ✓ |
| **ESP32-C3** | **✗** | ✗ | ✓ |
| ESP32-C6 / H2 | ✗ | ✗ | ✓ |
| RP2040 | ✓（TinyUSB） | ✗ | ✗（需外挂） |

要点：

- ESP32-C3 只有 "USB Serial/JTAG" 固定功能外设，**不能当任意 USB 设备**；且只有 BLE、没有蓝牙经典。所以 C3 做不了 USB DS4，也做不了无线 DS4（DS4 无线走 BR/EDR HID）。官方 esp-usb 文档也只覆盖 S2/S3/P4。
- 想做 **USB 版 DualShock 4**：RP2040（推荐，GP2040-CE 的 PS4 mode 已验证）或 ESP32-S3（TinyUSB，自行实现）。
- 想做**无线 DS4**：需要蓝牙经典，即原版 ESP32（但没有 USB 外设）。

## 6. chiaki-ng 的输入架构（源码结论）

- 手柄输入走 **SDL**，`CHIAKI_GUI_ENABLE_SDL_GAMECONTROLLER` 默认 ON；macOS 构建会打包 SDL。
- 初始化时设 `SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1")`，事件处理处判断 `if(joystick_allow_background_events || is_app_active)`，而设置项 `settings/allow_joystick_background_events` **默认 true** → **后台/最小化仍处理手柄输入**。
- 键盘输入走 `QmlMainWindow::event()` 的 `QEvent::KeyPress/KeyRelease` → **需要窗口有焦点**，最小化收不到。
- 所有输入源在 `StreamSession::SendFeedbackState()` 里用 `chiaki_controller_state_or()` 合并（按键按位或、扳机取最大、摇杆取最大幅值）。
- `cli/` 只有 `wakeup` 和 `discover`，**没有输入能力**。
- 核心库 `libchiaki` 提供 `chiaki_session_set_controller_state()`，`ChiakiConnectInfo` 里还有 `audio_video_disabled` —— 可以写纯输入客户端，但要自己处理配对/PSN/串流。

## 7. 构建配方（本机已验证）

### 7.1 libvirtualhid（macOS）

```sh
cmake -S third_party/libvirtualhid -B build/cpp \
  -DCMAKE_OSX_ARCHITECTURES='arm64;x86_64' \
  -DCMAKE_OSX_DEPLOYMENT_TARGET=14.2 \
  -DCMAKE_BUILD_TYPE=Release \
  -DBUILD_TESTS=OFF -DBUILD_DOCS=OFF -DBUILD_EXAMPLES=OFF \
  -DLIBVIRTUALHID_BUILD_TOOLS=OFF -DLIBVIRTUALHID_BUILD_MACOS_BROKER=OFF
```

坑：必须设 `CMAKE_OSX_DEPLOYMENT_TARGET >= 13`（推荐 14.2）。用 11.0 时 libc++ 会把 `std::format` / 浮点 `std::to_chars` 标记为不可用，编译报错。

### 7.2 chiaki-ng（macOS，带本仓库补丁）

```sh
brew install --force qt@6 ffmpeg pkg-config opus openssl@3 cmake ninja nasm \
  protobuf@29 speexdsp libplacebo json-c miniupnpc libevent sdl2
pip install 'protobuf>=5,<6'
git submodule update --init --depth 1 -- third-party/nanopb third-party/jerasure \
  third-party/gf-complete third-party/curl

export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig:/opt/homebrew/opt/ffmpeg/lib/pkgconfig"
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DCHIAKI_ENABLE_CLI=OFF -DCHIAKI_ENABLE_TESTS=OFF \
  -DCHIAKI_ENABLE_STEAMDECK_NATIVE=OFF -DCHIAKI_ENABLE_STEAM_SHORTCUT=OFF \
  -DOPENSSL_ROOT_DIR="$(brew --prefix)/opt/openssl@3" \
  -DCMAKE_PREFIX_PATH="$(brew --prefix)/opt/openssl@3;$(brew --prefix)/opt/qt@6;$(brew --prefix)/opt/protobuf@29"
cmake --build build --target chiaki
```

坑：`PKG_CONFIG_PATH` 不设就找不到 ffmpeg；`-DCHIAKI_ENABLE_TESTS=OFF` 省掉 munit 子模块；`-DCHIAKI_ENABLE_STEAM_SHORTCUT=OFF` 省掉 cpp-steam-tools。

## 8. 复用价值最高的三条

1. **虚拟手柄在 macOS 上是"许可问题"不是"技术问题"**：要先确认 broker/license/权限三件事，再谈实现。
2. **手柄输入与焦点无关，键盘输入与焦点强相关**：任何"最小化也要能发"的需求，都必须走手柄（或系统级注入）而非键盘。
3. **要"一个双人都能用的假 DS4"**：成本最低的是 RP2040 + GP2040-CE 的 PS4 mode；纯软件则只有"买 license"或"改客户端"。
