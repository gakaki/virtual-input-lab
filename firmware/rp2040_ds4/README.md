# RP2040 DualShock 4 桥接固件

让一块 RP2040 板子在 Mac 上**以真 DualShock 4 的身份出现**，你的 Python / Rust 代码通过同一根 USB 线（CDC 串口）发按键。

```
你的代码  --USB串口-->  RP2040  --USB HID-->  macOS 认为是 DualShock 4
   ↑                                          ↑
 打包 DS4 报文                              游戏/系统直接认
（用 libvirtualhid 已验证的打包逻辑）
```

一根线同时承载两个接口：**DualShock 4 手柄**（HID）+ **命令串口**（CDC）。板子插上 Mac 后，系统里会同时出现一个"无线控制器"和一个 `/dev/tty.usbmodem*`。

## 为什么这样可行

macOS 15+ 用代码凭空造虚拟手柄需要 Apple 受限 entitlement（见仓库根 README 的实测结论）。RP2040 方案绕开的不是这个限制，而是它的适用范围：**这是一块真实的 USB 设备**，macOS 认为它就是硬件手柄，不涉及虚拟设备。

## 需要什么

- 一块 RP2040 板：Raspberry Pi Pico、Pico 2（RP2350 也可以，改 `PICO_BOARD`）或任意 RP2040 兼容板
- 一条 USB 数据线（要能传数据，不是纯充电线）
- Pico SDK + ARM 工具链（一次性）

## 1. 一次性准备

```sh
# Pico SDK（含子模块，tinyusb 在其中）
git clone --depth 1 --recurse-submodules https://github.com/raspberrypi/pico-sdk.git ~/pico-sdk
export PICO_SDK_PATH=~/pico-sdk

# ARM 工具链（任选其一）
brew install arm-none-eabi-gcc cmake ninja
# 或从 ARM 官网下载 arm-gnu-toolchain，把 bin 加进 PATH
```

## 2. 构建与烧录

```sh
cd firmware/rp2040_ds4
cp "$PICO_SDK_PATH/external/pico_sdk_import.cmake" .

cmake -B build -G Ninja -DCMAKE_BUILD_TYPE=Release
cmake --build build

# 产物
ls build/rp2040_ds4.uf2
```

烧录：按住板子的 **BOOTSEL** 键插上 USB，会挂载一个 `RPI-RP2` 盘，把 `build/rp2040_ds4.uf2` 拖进去，板子自动重启。

## 3. 在 Mac 上确认设备出现

```sh
# 应当是 DualShock 4（Sony 054c:05c4）
system_profiler SPUSBDataType | grep -i -A8 sony
ioreg -r -c IOHIDDevice -l | grep -iE "Wireless Controller|054c"

# 串口
ls /dev/tty.usbmodem*
```

## 4. 用代码发按键

Python：

```sh
uv pip install pyserial
python python/examples/rp2040_send_all_buttons.py          # 自动找串口
python python/examples/rp2040_send_all_buttons.py --port /dev/tty.usbmodem1101
```

Rust：

```sh
cargo run -p rp2040-ds4-client
cargo run -p rp2040-ds4-client -- --port /dev/tty.usbmodem1101
```

两个客户端都会：按住全部 21 个逻辑按键 + 双扳机到底，保持 0.5 秒，然后松开。报文由 `libvirtualhid` 打包（就是虚拟后端用的同一份代码），所以格式不会跑偏。

## 协议

固件只做一件简单的事：把串口收到的载荷原样当作 DS4 输入报文转发。载荷是 `pack_dualshock4_report(..., usb=True)` 的输出**去掉首字节的报告 ID**（63 字节），TinyUSB 会补回报告 ID，线上正好是 64 字节。

一帧 67 字节：

```
A5 5A <len> <payload...> <xor>
 len     = 63
 payload = 63 字节 DS4 报文载荷
 xor     = len 与所有 payload 字节的异或
```

行为：

- 固件每 4 ms 重复发送当前报告（真手柄也是持续上报的）。
- **1 秒**收不到新帧就自动回到"全部松开"，避免卡键。
- 校验失败/长度不对的帧被丢弃并计数，下一帧可正常恢复。

## 自检（不需要硬件）

```sh
# 描述符结构校验（字节级解析，检查 wTotalLength/接口/端点）
python3 tools/check_usb_descriptors.py

# 固件源码语法检查 + 协议解析器单元测试
bash firmware/rp2040_ds4/host_test/check_syntax.sh
```

## 已知限制与注意

- **VID/PID 用的是 Sony 的 054C:05C4。** 这是 macOS 把它认成 DualShock 4 的前提（GameController 按 VID/PID 匹配），但该标识属于 Sony，仅供个人互操作使用；公开发布前请自行评估法律风险，或改用自己的 VID/PID（代价是系统不再把它当 DS4）。
- 触摸板、陀螺仪、加速度计字段会按报文携带，但固件不产生真实传感器数据；由主机端载荷决定（默认是静止值）。
- 输出报告（震动 / 灯条，报告 ID 0x05）已被接收但未驱动电机或 LED。要接电机/LED，在 `main.c` 的 `tud_hid_set_report_cb` 里消费 `buffer[3..7]` 即可。
- 特征报告（校准 0x02、配对 0x12 等）统一返回全 0。有线手柄不需要真实值即可被 macOS 接受；若某些游戏要求校准数据，需要按真实 DS4 的格式填。
- 固件本身的 TinyUSB/SDK 胶水层**未在本机编译过**（本机没有 ARM 工具链），但描述符结构、协议解析、以及主机端报文都已验证。第一次在你的机器上 `cmake --build` 时如有报错，多半是 SDK 版本/目标名差异，报给我即可。

## 文件

| 文件 | 作用 |
| --- | --- |
| `main.c` | 主循环：收串口帧、发 HID 报告、超时释放 |
| `ds4_frame.c/.h` | 帧解析器（无 SDK 依赖，可主机端单测） |
| `usb_descriptors.c` | 复合 USB 描述符（DS4 HID + CDC） |
| `tusb_config.h` | TinyUSB 配置 |
| `generated/ds4_usb.h` | 由 `cargo run -p livvirtualhid --example dump_ds4` 生成：DS4 身份、467 字节描述符、中性/全按键载荷、按键位定义 |
| `host_test/` | 语法检查脚本、解析器单测、串口帧解码器 |
