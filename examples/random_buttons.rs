use std::time::Duration;

use virtual_gamepad::*;

fn main() {
    let mut gamepad = VirtualGamepad::new(GamepadType::Xbox360).unwrap();
    for _ in 0..256 {
        let button = GamepadButton::from_u8((getrandom::u32().unwrap() % 15) as u8).unwrap();
        let state = getrandom::u32().unwrap() % 1 == 0;
        gamepad.update(GamepadUpdate::Button {
            button,
            pressed: state,
        });
        std::thread::sleep(Duration::from_millis(500));
    }
}
