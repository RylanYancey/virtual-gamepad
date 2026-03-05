
# Virtual Gamepads
This crate can be used to create artificial input devices.

# Supported Platforms
Currently only Linux is supported, but Windows and MacOs support is planned.
 - [x] Linux
 - [ ] Windows
 - [ ] MacOs
 
 # Usage
```rs
use virtual_gamepad::{VirtualGamepad, GamepadType};

fn main() {
    let mut gamepad = VirtualGamepad::new(GamepadType::Xbox360);
}
```
