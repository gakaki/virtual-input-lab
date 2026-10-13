# chiaki-ng 本地输入源补丁

给 chiaki-ng 加一个**本地 socket 输入源**：外部程序（Python / Rust / 任何能发 Unix 数据报的东西）把 PS4 手柄状态直接送进串流会话。

```
你的 Python / Rust 代码 --Unix 数据报--> chiaki-ng --PS Remote Play--> PS4 / PS5
```

**为什么最小化也生效**：这条路径完全不经过窗口系统。补丁把外部状态并进 chiaki-ng 已有的输入合并点（`SendFeedbackState()` 里的 `chiaki_controller_state_or`），和"键盘必须有焦点"那条路无关。

## 文件

- `chiaki-ng-socket-input.patch` — 补丁本体（3 个文件，+231 行，**不需要改 CMake**）
- `socketinput_protocol.h` — 协议头的仓库副本，与补丁内嵌版本逐字节一致（脚本会校验）

补丁基线：chiaki-ng `master` @ `c8bea26`，已在全新克隆上验证 `git apply --check` 通过。

## 1. 打补丁

```sh
git clone https://github.com/streetpea/chiaki-ng.git
cd chiaki-ng
git apply /path/to/chiaki-ng-socket-input.patch
```

补丁改了这些地方：

- 新增 `gui/include/socketinput_protocol.h`（协议定义，纯 C，不依赖 Qt）
- `gui/include/streamsession.h`：新增成员与槽
- `gui/src/streamsession.cpp`：建立/销毁 socket、`QSocketNotifier` 收包、1 秒无包自动释放、以及一行合并

## 2. 在 macOS 上编译

依赖与 chiaki-ng 官方 macOS CI 一致：

```sh
brew install --force qt@6 ffmpeg pkgconfig opus openssl@3 cmake ninja nasm \
  protobuf@29 speexdsp libplacebo wget python-setuptools json-c miniupnpc libevent

git clone https://github.com/libsdl-org/sdl2-compat.git
sudo INSTALL_PREFIX=/usr/local ./scripts/build-sdl2-compat.sh
sudo cp /usr/local/lib/pkgconfig/sdl2-compat.pc /usr/local/lib/pkgconfig/sdl2.pc

cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DCHIAKI_ENABLE_CLI=OFF -DCHIAKI_ENABLE_STEAMDECK_NATIVE=OFF \
  -DOPENSSL_ROOT_DIR="$(brew --prefix)/opt/openssl@3" \
  -DCMAKE_PREFIX_PATH="$(brew --prefix)/opt/openssl@3;$(brew --prefix)/opt/qt@6;$(brew --prefix)/opt/protobuf@29"
export CPATH="$(brew --prefix)/opt/ffmpeg/include"
cmake --build build --target chiaki
```

产物在 `build/gui/chiaki-ng.app`。开始串流后日志里应出现：

```
Socket input: listening on /tmp/chiaki-ng-input.sock
```

没出现就说明补丁没生效。

## 3. 协议

每个状态一条数据报，固定 **24 字节**、小端、无隐式填充（定义见 `socketinput_protocol.h`）：

```
偏移  类型  字段
 0    u32   magic = 0x4B504943 ("CIPK")
 4    u16   version = 1
 6    u16   size = 24
 8    u32   buttons 位掩码
12    u8    L2 模拟量 0..255
13    u8    R2 模拟量 0..255
14    i16   左摇杆 X  (-0x7fff..0x7fff, 0 居中)
16    i16   左摇杆 Y
18    i16   右摇杆 X
20    i16   右摇杆 Y
22    u16   保留
```

按钮位（对应 `ChiakiControllerButton`）：

```
0x0001 cross      0x0100 L1
0x0002 circle     0x0200 R1
0x0004 square     0x0400 L3
0x0008 triangle   0x0800 R3
0x0010 十字键左    0x1000 options
0x0020 十字键右    0x2000 share
0x0040 十字键上    0x4000 touchpad
0x0080 十字键下    0x8000 PS
```

全按键 = `0xFFFF`。

行为：

- socket 路径默认 `/tmp/chiaki-ng-input.sock`，可用环境变量 `CHIAKI_NG_INPUT_SOCKET` 覆盖（服务端和客户端要一致）。
- 每收到一条就立刻推一次状态；**1 秒**没收到新包就把该输入源清零，避免卡键。
- 合并语义沿用 chiaki-ng 的 `chiaki_controller_state_or`：按键按位或、扳机取最大、摇杆按最大幅值——所以外部输入能和真实手柄/键盘共存。

## 4. 用法

Python：

```sh
python python/examples/chiaki_send_buttons.py                        # 全部按键按 0.5 秒
python python/examples/chiaki_send_buttons.py --buttons cross r1 l2  # 指定按键
python python/examples/chiaki_send_buttons.py --socket /tmp/xxx.sock
```

Rust：

```sh
cargo run -p chiaki-socket-client
cargo run -p chiaki-socket-client -- --socket /tmp/xxx.sock
```

## 5. 自检（不需要编译 chiaki-ng）

### 5.1 真实构建与真机测试（已做）

本补丁已在 macOS 上**真实编译并运行验证**：

- chiaki-ng @ `c8bea26`，Qt 6.11.2 / ffmpeg 9.0.2 / sdl2-compat 2.32.74 / openssl@3 3.6.1
- 产物 `build/gui/chiaki.app`，符号含 `ControllerManager::StartSocketInput/StopSocketInput/PollSocketInput`
- 启动后日志出现 `Socket input: listening on /tmp/chiaki-ng-input.sock`

外部客户端发送 → chiaki-ng 实际收到的日志：

```
# Rust 客户端（全部按键 0xffff）
Socket input: buttons=0xffff l2=255 r2=255 left=(0,0) right=(0,0)
Socket input: buttons=0x0000 l2=0   r2=0   left=(0,0) right=(0,0)
# Python 客户端（cross+circle+square+triangle = 0x000f）
Socket input: buttons=0x000f l2=255 r2=255 left=(0,0) right=(0,0)
Socket input: buttons=0x0000 l2=0   r2=0   left=(0,0) right=(0,0)
```

**最小化验证**：把 chiaki-ng 窗口最小化（辅助功能属性 `AXMinimized = true`）后再次发送，日志同样打出 `buttons=0xffff` / `buttons=0x000f` —— 最小化不影响接收。

构建时踩过的坑（都在第 2 节命令里规避了）：

- 必须设 `PKG_CONFIG_PATH=/opt/homebrew/lib/pkgconfig:/opt/homebrew/opt/ffmpeg/lib/pkgconfig`，否则找不到 ffmpeg
- 需要子模块 `third-party/nanopb`、`third-party/jerasure`、`third-party/gf-complete`、`third-party/curl`
- 生成 nanopb 代码需要 Python 的 `protobuf` 5.x（`pip install 'protobuf>=5,<6'`）
- 建议 `-DCHIAKI_ENABLE_TESTS=OFF -DCHIAKI_ENABLE_STEAM_SHORTCUT=OFF`，省掉 munit / cpp-steam-tools 子模块

```sh
bash tools/test_chiaki_socket_input.sh
```

它会编译一个**使用同一份协议头**的参考接收器，分别接收 Python / Rust 客户端的包并解码，再校验两个客户端字节一致、以及仓库副本与补丁内嵌副本一致。

本机实测：

```
python: held messages=9   last=[buttons=0x0000 l2=0 r2=0 ...]   rejected=0
rust:   held messages=21  last=[buttons=0x0000 l2=0 r2=0 ...]   rejected=0
clients agree byte for byte (96 bytes)
protocol header matches the patch
ALL CHECKS PASSED
```

## 6. 已知限制

- 补丁已在 macOS 上编译并真机验证（见 5.1）；`git apply --check` 也在全新克隆上验证过。唯一没测的是**真实 PS4/PS5 串流**（需要主机和 PSN 会话）。
- **没有触摸板/陀螺仪**：当前只覆盖按键、双扳机、双摇杆；协议里预留了扩展位。
- **仍需真实 PS4/PS5 会话**：先在 chiaki-ng 里正常连上并开始串流，补丁只负责把输入接进去，不解决配对/登录。
- **AGPL-3.0**：chiaki-ng 是 AGPL。自己用没问题；对外分发修改版要按 AGPL 提供源码。
- 若最小化后仍不动，检查 App Nap：`defaults write org.streetpea.chiaking NSAppSleepDisabled -bool YES`。
