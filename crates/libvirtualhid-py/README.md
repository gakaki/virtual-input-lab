# libvirtualhid (Python)

PyO3 bindings for the `livvirtualhid` Rust crate, which wraps the
[libvirtualhid](https://github.com/LizardByte/libvirtualhid) virtual HID library.

```python
import libvirtualhid as lvh

runtime = lvh.Runtime()             # in-memory backend, works everywhere
gamepad = runtime.create_dualshock4(usb=False)
state = lvh.GamepadState.all_buttons()
gamepad.submit(state)
print(gamepad.last_input_report().hex(" "))
```

On macOS, `lvh.Runtime(platform=True)` needs the signed, licensed Virtual HID
Broker and Apple's virtual HID entitlement before gamepad creation succeeds.
