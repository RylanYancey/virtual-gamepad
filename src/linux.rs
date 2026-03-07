use libc::{input_event, timeval};
use nix::unistd::write;
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr};
use std::fs::File;
use std::io;
use std::mem::zeroed;
use std::os::fd::AsRawFd;
use std::time::Duration;

use crate::GamepadButton;

const UINPUT_PATH: &str = "/dev/uinput";

// ioctl definitions
// This will generate functions for the raw C functions.
ioctl_write_int!(ui_set_evbit, b'U', 100);
ioctl_write_int!(ui_set_keybit, b'U', 101);
ioctl_write_int!(ui_set_absbit, b'U', 103);
ioctl_none!(ui_dev_create, b'U', 1);
ioctl_none!(ui_dev_destroy, b'U', 2);
ioctl_write_ptr!(ui_dev_setup, b'U', 3, libc::uinput_setup);
ioctl_write_ptr!(ui_abs_setup, b'U', 4, libc::uinput_abs_setup);

/// Writes an input event for a gamepad.
fn emit(file: &mut File, type_: u16, code: u16, value: i32) {
    let mut ev: input_event = unsafe { zeroed() };
    ev.time = timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    ev.type_ = type_;
    ev.code = code;
    ev.value = value;

    let bytes = unsafe {
        std::slice::from_raw_parts(
            &ev as *const input_event as *const u8,
            std::mem::size_of::<input_event>(),
        )
    };

    write(file, bytes).unwrap();
}

// Numbers that identify bus types in linux.
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input.h#L256
const BUS_USB: u16 = 0x03;

pub type ParamType = nix::sys::ioctl::ioctl_param_type;

// Numbers that identify input event types in linux
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const EV_KEY: ParamType = 0x01;
const EV_ABS: ParamType = 0x03;
const EV_SYN: ParamType = 0x00; // Synchronize input

// Numbers that identify specific buttons and keys in linux.
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const BTN_SOUTH: ParamType = 0x130;
const BTN_NORTH: ParamType = 0x133;
const BTN_WEST: ParamType = 0x134;
const BTN_EAST: ParamType = 0x131;
const BTN_DPAD_UP: ParamType = 0x220;
const BTN_DPAD_DOWN: ParamType = 0x221;
const BTN_DPAD_LEFT: ParamType = 0x222;
const BTN_DPAD_RIGHT: ParamType = 0x223;
const BTN_SELECT: ParamType = 0x13a;
const BTN_START: ParamType = 0x13b;
const BTN_MODE: ParamType = 0x13c;
const BTN_THUMBL: ParamType = 0x13d;
const BTN_THUMBR: ParamType = 0x13e;
const BTN_TRIGGER_LEFT: ParamType = 0x136; // bumper left
const BTN_TRIGGER_RIGHT: ParamType = 0x137; // bumper right
const BTN_TRIGGER_LEFT2: ParamType = 0x138; // maybe analog
const BTN_TRIGGER_RIGHT2: ParamType = 0x139; // maybe analog

// Numbers that identify absolute axes inputs in linux.
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const LEFT_STICK_X: ParamType = 0x00; // ABS_X
const LEFT_STICK_Y: ParamType = 0x01; // ABS_Y
const RIGHT_STICK_X: ParamType = 0x03; // ABS_RX
const RIGHT_STICK_Y: ParamType = 0x04; // ABS_RY
const ANALOG_TRIGGER_LEFT: ParamType = 0x02; // ABS_Z
const ANALOG_TRIGGER_RIGHT: ParamType = 0x05; // ABS_RZ

pub(super) struct RawGamepad {
    file: File,
}

impl RawGamepad {
    pub fn new(vendor: u16, product: u16, name: &str) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new().write(true).open(UINPUT_PATH)?;
        let fd = file.as_raw_fd(); // raw unix functions expect file ptr as i32.

        // Configure supported event types
        unsafe {
            // Adds support for keys and buttons
            ui_set_evbit(fd, EV_KEY)?;
            // adds support for Absolute Axis inputs (analog triggers / axes)
            ui_set_evbit(fd, EV_ABS)?;
        }

        // Configure supported keys/buttons
        unsafe {
            let keybits = &[
                BTN_SOUTH,
                BTN_NORTH,
                BTN_WEST,
                BTN_EAST,
                BTN_DPAD_UP,
                BTN_DPAD_DOWN,
                BTN_DPAD_LEFT,
                BTN_DPAD_RIGHT,
                BTN_SELECT,
                BTN_START,
                BTN_MODE,
                BTN_THUMBL,
                BTN_THUMBR,
                BTN_TRIGGER_LEFT,
                BTN_TRIGGER_RIGHT,
                BTN_TRIGGER_LEFT2,
                BTN_TRIGGER_RIGHT2,
            ];

            for keybit in keybits {
                ui_set_keybit(fd, *keybit)?;
            }
        }

        // Configure supported Absolute Axes and analog triggers.
        unsafe {
            // Sticks
            let absbits = &[LEFT_STICK_X, LEFT_STICK_Y, RIGHT_STICK_X, RIGHT_STICK_Y];
            for absbit in absbits {
                ui_set_absbit(fd, *absbit)?;
                let abs_setup = libc::uinput_abs_setup {
                    code: *absbit as u16,
                    absinfo: libc::input_absinfo {
                        minimum: -32768,
                        maximum: 32767,
                        flat: 0,
                        fuzz: 0,
                        resolution: 0,
                        value: 0,
                    },
                };
                ui_abs_setup(fd, &abs_setup)?;
            }

            // Analog triggers
            let triggerbits = &[ANALOG_TRIGGER_LEFT, ANALOG_TRIGGER_RIGHT];
            for triggerbit in triggerbits {
                ui_set_absbit(fd, *triggerbit)?;
                let abs_setup = libc::uinput_abs_setup {
                    code: *triggerbit as u16,
                    absinfo: libc::input_absinfo {
                        minimum: 0,
                        maximum: 32767,
                        flat: 0,
                        fuzz: 0,
                        resolution: 0,
                        value: 0,
                    },
                };
                ui_abs_setup(fd, &abs_setup)?;
            }
        }

        // Give the OS information about the device.
        unsafe {
            let mut usetup: libc::uinput_setup = zeroed();
            usetup.id.bustype = BUS_USB;
            usetup.id.vendor = vendor;
            usetup.id.product = product;
            // write bytes of name as i8, cuz reasons??
            let name_bytes = name.as_bytes();
            for (i, b) in name_bytes.iter().enumerate() {
                usetup.name[i] = *b as i8;
            }
            // Make sure name is null-terminated.
            usetup.name[name_bytes.len()] = 0;
            ui_dev_setup(fd, &usetup)?;
        }

        // Create the device
        unsafe {
            ui_dev_create(fd)?;
        }

        // This is needed because the OS will not wait for device creation to complete.
        std::thread::sleep(Duration::from_millis(500));

        Ok(Self { file })
    }

    pub fn update(&mut self, button: GamepadButton, values: [f32; 2]) {
        match button {
            _ if button.is_joystick() => {
                let (x_code, y_code) = match button {
                    GamepadButton::LeftStick => (LEFT_STICK_X, LEFT_STICK_Y),
                    GamepadButton::RightStick => (RIGHT_STICK_X, RIGHT_STICK_Y),
                    _ => unreachable!(),
                };
                emit(
                    &mut self.file,
                    EV_ABS as u16,
                    x_code as u16,
                    crate::quantize(values[0]) as i32,
                );
                emit(
                    &mut self.file,
                    EV_ABS as u16,
                    y_code as u16,
                    crate::quantize(values[1]) as i32,
                );
            }
            _ if button.is_trigger() => {
                let code = match button {
                    GamepadButton::LeftTrigger => BTN_TRIGGER_LEFT2,
                    GamepadButton::RightTrigger => BTN_TRIGGER_RIGHT2,
                    _ => unreachable!(),
                };

                emit(
                    &mut self.file,
                    EV_ABS as u16,
                    code as u16,
                    crate::quantize(values[0]) as i32,
                );
            }
            _ => {
                let code = match button {
                    GamepadButton::South => BTN_SOUTH,
                    GamepadButton::North => BTN_NORTH,
                    GamepadButton::East => BTN_EAST,
                    GamepadButton::West => BTN_WEST,
                    GamepadButton::DPadUp => BTN_DPAD_UP,
                    GamepadButton::DPadDown => BTN_DPAD_DOWN,
                    GamepadButton::DPadLeft => BTN_DPAD_LEFT,
                    GamepadButton::DPadRight => BTN_DPAD_RIGHT,
                    GamepadButton::LeftBumper => BTN_TRIGGER_LEFT,
                    GamepadButton::RightBumper => BTN_TRIGGER_RIGHT,
                    GamepadButton::LeftThumb => BTN_THUMBL,
                    GamepadButton::RightThumb => BTN_THUMBR,
                    GamepadButton::Start => BTN_START,
                    GamepadButton::Select => BTN_SELECT,
                    GamepadButton::Mode => BTN_MODE,
                    _ => unreachable!(),
                };
                emit(
                    &mut self.file,
                    EV_KEY as u16,
                    code as u16,
                    (values[0] < 0.5) as i32,
                );
            }
        };
        emit(&mut self.file, EV_SYN as u16, 0, 0);
    }
}

impl Drop for RawGamepad {
    fn drop(&mut self) {
        unsafe {
            // I don't unwrap this because unwrapping in a destructor is bad.
            let _ = ui_dev_destroy(self.file.as_raw_fd());
        }
    }
}
