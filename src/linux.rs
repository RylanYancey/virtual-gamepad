use libc::{input_event, timeval};
#[cfg(feature = "rumble")]
use nix::ioctl_readwrite;
use nix::unistd::write;
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr};
use std::fs::File;
use std::io;
use std::mem::zeroed;
use std::os::fd::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::RumbleState;
use crate::state::{Buttons, GamepadState};

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

// Rumble related ioctl functions
#[cfg(feature = "rumble")]
ioctl_write_int!(ui_set_ffbit, b'U', 107);
#[cfg(feature = "rumble")]
ioctl_readwrite!(ui_begin_ff_upload, b'U', 200, libc::uinput_ff_upload);
#[cfg(feature = "rumble")]
ioctl_readwrite!(ui_end_ff_upload, b'U', 201, libc::uinput_ff_upload);
#[cfg(feature = "rumble")]
ioctl_readwrite!(ui_begin_ff_erase, b'U', 202, libc::uinput_ff_erase);
#[cfg(feature = "rumble")]
ioctl_write_ptr!(ui_end_ff_erase, b'U', 203, libc::uinput_ff_erase);

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
const EV_KEY: u16 = 0x01;
const EV_ABS: u16 = 0x03;
const EV_SYN: u16 = 0x00; // Synchronize input
const EV_FF: u16 = 0x15; // Receive rumble (why is this FF?)
const EV_UINPUT: u16 = 0x0101; // Used to receive uinput events
const UI_FF_UPLOAD: u16 = 1;
const UI_FF_ERASE: u16 = 2;

// Numbers that identify specific buttons and keys in linux.
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const BTN_SOUTH: u16 = 0x130;
const BTN_NORTH: u16 = 0x133;
const BTN_WEST: u16 = 0x134;
const BTN_EAST: u16 = 0x131;
const BTN_SELECT: u16 = 0x13a;
const BTN_START: u16 = 0x13b;
const BTN_MODE: u16 = 0x13c;
const BTN_THUMBL: u16 = 0x13d;
const BTN_THUMBR: u16 = 0x13e;
const BTN_TRIGGER_LEFT: u16 = 0x136; // bumper left
const BTN_TRIGGER_RIGHT: u16 = 0x137; // bumper right

// Numbers that specify Rumble otuput in linux.
const FF_RUMBLE: u16 = 0x50;

// Numbers that identify absolute axes inputs in linux.
// From: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
const LEFT_STICK_X: u16 = 0x00; // ABS_X
const LEFT_STICK_Y: u16 = 0x01; // ABS_Y
const RIGHT_STICK_X: u16 = 0x03; // ABS_RX
const RIGHT_STICK_Y: u16 = 0x04; // ABS_RY
const TRIGGER_LEFT: u16 = 0x02; // ABS_Z
const TRIGGER_RIGHT: u16 = 0x05; // ABS_RZ
const DPAD_X: u16 = 0x10; // ABS_HAT0X
const DPAD_Y: u16 = 0x11; // ABS_HAT0Y

pub(super) struct RawGamepad {
    file: File,
    prev: GamepadState,
    #[cfg(feature = "rumble")]
    rumble: Arc<AtomicU32>,
}

impl RawGamepad {
    pub fn new(vendor: u16, product: u16, name: &str) -> io::Result<Self> {
        let file = std::fs::OpenOptions::new().write(true).open(UINPUT_PATH)?;
        let fd = file.as_raw_fd(); // raw unix functions expect file ptr as i32.

        // Configure supported event types
        unsafe {
            // Adds support for keys and buttons
            ui_set_evbit(fd, EV_KEY as ParamType)?;
            // adds support for Absolute Axis inputs (analog triggers / axes)
            ui_set_evbit(fd, EV_ABS as ParamType)?;
        }

        // Configure supported keys/buttons
        unsafe {
            let keybits = &[
                BTN_SOUTH,
                BTN_NORTH,
                BTN_WEST,
                BTN_EAST,
                BTN_SELECT,
                BTN_START,
                BTN_MODE,
                BTN_THUMBL,
                BTN_THUMBR,
                BTN_TRIGGER_LEFT,
                BTN_TRIGGER_RIGHT,
            ];

            for keybit in keybits {
                ui_set_keybit(fd, *keybit as ParamType)?;
            }
        }

        // Configure supported Absolute Axes and analog triggers.
        unsafe {
            // Sticks (i16 axes)
            let absbits = &[LEFT_STICK_X, LEFT_STICK_Y, RIGHT_STICK_X, RIGHT_STICK_Y];
            for absbit in absbits {
                ui_set_absbit(fd, *absbit as ParamType)?;
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

            // DPads (analog i8)
            let dpadbits = &[DPAD_X, DPAD_Y];
            for absbit in dpadbits {
                ui_set_absbit(fd, *absbit as ParamType)?;
                let abs_setup = libc::uinput_abs_setup {
                    code: *absbit,
                    absinfo: libc::input_absinfo {
                        minimum: -128,
                        maximum: 127,
                        flat: 0,
                        fuzz: 0,
                        resolution: 0,
                        value: 0,
                    },
                };
                ui_abs_setup(fd, &abs_setup)?;
            }

            // Analog triggers (analog u8)
            let triggerbits = &[TRIGGER_LEFT, TRIGGER_RIGHT];
            for triggerbit in triggerbits {
                ui_set_absbit(fd, *triggerbit as ParamType)?;
                let abs_setup = libc::uinput_abs_setup {
                    code: *triggerbit,
                    absinfo: libc::input_absinfo {
                        minimum: 0,
                        maximum: 255,
                        flat: 0,
                        fuzz: 0,
                        resolution: 0,
                        value: 0,
                    },
                };
                ui_abs_setup(fd, &abs_setup)?;
            }
        }

        // Enable rumble events.
        #[cfg(feature = "rumble")]
        unsafe {
            ui_set_evbit(fd, EV_FF as ParamType)?;
            ui_set_ffbit(fd, FF_RUMBLE as ParamType)?;
        }

        // Give the OS information about the device.
        unsafe {
            let mut usetup: libc::uinput_setup = zeroed();
            usetup.id.bustype = BUS_USB;
            usetup.id.vendor = vendor;
            usetup.id.product = product;
            usetup.ff_effects_max = 16;
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

        // Spawn rumble read thread.
        #[cfg(feature = "rumble")]
        let rumble = unsafe {
            let rumble = Arc::new(AtomicU32::new(0));
            let rumble2 = rumble.clone();
            std::thread::spawn(move || read_rumble_events(fd, rumble2));
            rumble
        };

        Ok(Self {
            file,
            prev: GamepadState::default(),
            #[cfg(feature = "rumble")]
            rumble,
        })
    }

    fn emit_if<I: Into<i32>>(&mut self, prev: I, new: I, ev: u16) {
        let (prev, new) = (prev.into(), new.into());
        if prev != new {
            emit(&mut self.file, EV_ABS as u16, ev, new);
        }
    }

    pub fn update(&mut self, state: GamepadState) {
        // early return if the state is not actually changed.
        if self.prev == state {
            self.prev = state;
            return;
        }

        // compute button changes
        let just_pressed = state.buttons.just_pressed(self.prev.buttons);
        let just_released = state.buttons.just_released(self.prev.buttons);
        let changed = just_pressed | just_released;

        // write simple button changes.
        for button in changed.iter() {
            let code = match button {
                Buttons::SOUTH => BTN_SOUTH,
                Buttons::NORTH => BTN_NORTH,
                Buttons::EAST => BTN_EAST,
                Buttons::WEST => BTN_WEST,
                Buttons::BUMPER_LEFT => BTN_TRIGGER_LEFT,
                Buttons::BUMPER_RIGHT => BTN_TRIGGER_RIGHT,
                Buttons::THUMB_LEFT => BTN_THUMBL,
                Buttons::THUMB_RIGHT => BTN_THUMBR,
                Buttons::START => BTN_START,
                Buttons::SELECT => BTN_SELECT,
                Buttons::MODE => BTN_MODE,
                _ => continue,
            };

            emit(
                &mut self.file,
                EV_KEY as u16,
                code as u16,
                // if not just pressed (1), then it was just released. (0)
                just_pressed.contains(button) as i32,
            );
        }

        // Write analog axis updates
        let prev = self.prev;
        self.emit_if(state.dpad_x, prev.dpad_x, DPAD_X);
        self.emit_if(state.dpad_y, prev.dpad_y, DPAD_Y);
        self.emit_if(state.trigger_left, prev.trigger_left, TRIGGER_LEFT);
        self.emit_if(state.trigger_right, prev.trigger_right, TRIGGER_RIGHT);
        self.emit_if(state.stick_left_x, prev.stick_left_x, LEFT_STICK_X);
        self.emit_if(state.stick_left_y, prev.stick_left_y, LEFT_STICK_Y);
        self.emit_if(state.stick_right_x, prev.stick_right_x, RIGHT_STICK_X);
        self.emit_if(state.stick_right_y, prev.stick_right_y, RIGHT_STICK_Y);

        // Inform linux of available events for this device.
        emit(&mut self.file, EV_SYN as u16, 0, 0);

        // update state
        self.prev = state;
    }

    #[cfg(feature = "rumble")]
    pub fn rumble(&self) -> RumbleState {
        let val = self.rumble.load(Ordering::Relaxed);
        RumbleState {
            strong: val as u16,
            weak: (val >> 16) as u16,
        }
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

#[cfg(feature = "rumble")]
unsafe fn read_rumble_events(fd: i32, shared: Arc<AtomicU32>) {
    unsafe {
        loop {
            let mut ev: libc::input_event = std::mem::zeroed();

            // Read the file as an input event.
            //
            // If an error exit code is returned, the device has
            // been closed, and we can safely exit the loop.
            //
            // In libc, error codes are negative.
            if libc::read(
                fd,
                &mut ev as *mut _ as *mut libc::c_void,
                std::mem::size_of::<libc::input_event>(),
            ) < 0
            {
                break;
            }

            if ev.type_ == EV_UINPUT {
                match ev.code {
                    UI_FF_UPLOAD => {
                        let mut upload: libc::uinput_ff_upload = std::mem::zeroed();
                        upload.request_id = ev.value as u32;
                        ui_begin_ff_upload(fd, &mut upload).unwrap();
                        if upload.effect.type_ == FF_RUMBLE as u16 {
                            let rumble: libc::ff_rumble_effect =
                                std::mem::transmute(upload.effect.u[0] as u32);
                            let val = rumble.strong_magnitude as u32
                                | ((rumble.weak_magnitude as u32) << 16);
                            shared.store(val, Ordering::Relaxed);
                            upload.retval = 0;
                        } else {
                            upload.retval = -1;
                        }
                        ui_end_ff_upload(fd, &mut upload).unwrap();
                    }
                    UI_FF_ERASE => {
                        let mut erase: libc::uinput_ff_erase = std::mem::zeroed();
                        erase.request_id = ev.value as u32;
                        ui_begin_ff_erase(fd, &mut erase).unwrap();
                        erase.retval = 0;
                        ui_end_ff_erase(fd, &erase).unwrap();
                        shared.store(0, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        }
    }
}
