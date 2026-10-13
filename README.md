# livvirtualhid-mac

把 [LizardByte/libvirtualhid](https://github.com/LizardByte/libvirtualhid)（跨平台 C++ 虚拟 HID 库）
在 macOS 上编译出来，并加上 Rust / Python 两套绑定，用来做一个"按键映射 → 虚拟 DualShock 4"的模块。

本仓库包含三层：

1. `third_party/libvirtualhid`：上游源码（**不随仓库分发**，见下方准备步骤）。
2. `crates/`：Rust 封装。
   - `libvirtualhid-sys`：编译上游 C++ 静态库，并提供一层扁平 C ABI（`shim/lvh_shim.{h,cpp}`）。
   - `livvirtualhid`：安全 Rust API + 按键映射（`keymap`）。
   - `libvirtualhid-py`：PyO3 绑定，产出 `import libvirtualhid`。
3. `python/`、`packaging/`、`scripts/`：示例、DMG 打包与安装脚本。

另外还有两块：

- `firmware/rp2040_ds4/`：RP2040 固件，把板子伪装成真 DualShock 4（详见其 README）。
- `patches/`：chiaki-ng 的本地 socket 输入源补丁 + Python/Rust 客户端（详见 `patches/README.md`）。
- 调研结论汇总在 [FINDINGS.md](FINDINGS.md)。

## 准备 vendored 依赖

上游 `libvirtualhid` 的 macOS broker 等源码是 LB-SAL 许可（禁止再分发），因此本仓库不打包它，需要自行拉取：

```sh
git clone https://github.com/LizardByte/libvirtualhid.git third_party/libvirtualhid
git -C third_party/libvirtualhid checkout 006dccc8c508e2a54495d0b1a5b277519b146fe4
```

## 快速开始

```sh
# 一次性准备：cmake + maturin（走清华镜像）
uv venv --python 3.12 .venv
UV_DEFAULT_INDEX=https://pypi.tuna.tsinghua.edu.cn/simple uv pip install --python .venv/bin/python cmake maturin

# 编译 C++ 库 + Rust + Python wheel + DMG
bash scripts/build.sh
bash scripts/package_dmg.sh --no-build
```

产物：

| 文件 | 说明 |
| --- | --- |
| `build/cpp-universal/src/libvirtualhid.a` | 通用（arm64 + x86_64）C++ 静态库 |
| `target/release/examples/send_all_buttons` | Rust 示例二进制 |
| `dist/libvirtualhid-0.1.0-cp39-abi3-macosx_14_0_arm64.whl` | Python 扩展 wheel |
| `dist/libvirtualhid-macOS-arm64.dmg` | 打包好的安装镜像 |

## 发送一次 DualShock 4 全部按键

Rust：

```sh
CMAKE="$PWD/.venv/bin/cmake" cargo run -p livvirtualhid --example send_all_buttons
```

Python（先装 wheel：`uv pip install --python .venv/bin/python dist/*.whl`）：

```sh
.venv/bin/python python/examples/send_all_buttons.py
```

两者输出一致：创建 DualShock 4 → 按住全部 21 个逻辑按键（外加双扳机到底）→ 打印 78B（蓝牙）
与 64B（USB）两份 HID 输入报文 → `submit()` → 读回 `last_input_report()`。

## 两种后端

| 后端 | 构造方式（Rust / Python） | 说明 |
| --- | --- | --- |
| `fake`（默认） | `Runtime::new()` / `Runtime()` | 内存后端，任何机器都能完整跑通 create→submit→报文回读 |
| `platform` | `Runtime::platform()` / `Runtime(platform=True)` | 原生后端 |

`platform` 后端在 macOS 上的能力查询会显示 `supports_virtual_hid = true`，但真正创建虚拟手柄需要同时满足：

1. 安装上游**已签名的 Virtual HID Broker**（LaunchDaemon 系统服务）；
2. 该 broker 带 Apple 审批的 `com.apple.developer.hid.virtual.device` entitlement；
3. 机器上**已激活有效 license**。

本机当前没有这些条件，所以 `create_dualshock4()` 会返回：

```
backend_unavailable: macOS broker directory is missing or insecure
```

这是上游 macOS 方案的硬限制，不是本仓库代码的问题；`com.apple.developer.hid.virtual.device` 属于
Apple 受限 entitlement，无法自行签发。因此本仓库交付的是：**可编译、可运行的库与绑定，加上在任意
平台都能验证的完整数据链路**；真实虚拟设备创建需再补上上游签名 broker 与 license。

## 按键映射

`livvirtualhid::keymap` / `libvirtualhid.GamepadState.from_keys([...])` 提供默认映射表，
覆盖 DualShock/DualSense 名称（`cross`/`circle`/`square`/`triangle`/`l1`/`r2`…）、Xbox 名称
（`a`/`lb`/`rb`…）、以及少量键盘键位。模拟扳机名（`l2`/`r2`）会直接拉满对应扳机值。

```python
import libvirtualhid as lvh
state = lvh.GamepadState.from_keys(["cross", "r1", "dpad_up", "l2"])
print(state.pressed(), state.left_trigger)
```

## RP2040 硬件方案（不花钱的真 DualShock 4）

macOS 15+ 用代码凭空造虚拟手柄需要 Apple 受限 entitlement（实测：未签名返回 NULL，自签名带 entitlement 被 AMFI 直接 SIGKILL）。绕开的办法是**用真硬件**：让一块 RP2040 以真 DualShock 4 的身份插到 Mac 上。

`firmware/rp2040_ds4/` 是完整固件；一台 Mac 上一根 USB 线同时提供「DualShock 4 手柄 + 命令串口」：

```
你的 Python/Rust 代码 --串口--> RP2040 --USB HID--> macOS 认为是 DualShock 4
```

- 报文用本仓库的 `pack_dualshock4_report` 打包（和虚拟后端同一份逻辑），固件只负责转发，格式不会跑偏
- 主机端：`python/examples/rp2040_send_all_buttons.py`（pyserial）或 `cargo run -p rp2040-ds4-client`
- 详情、烧录步骤、协议、已知限制见 [firmware/rp2040_ds4/README.md](firmware/rp2040_ds4/README.md)

## 许可

- 上游 C++ 库：MIT（见 `third_party/libvirtualhid/LICENSES/MIT.md`）。
- 上游 Windows/macOS broker 与专有源码：LB-SAL 1.0，**未包含**在本仓库构建产物中。
- 本仓库新增的 Rust/Python 绑定：MIT。
