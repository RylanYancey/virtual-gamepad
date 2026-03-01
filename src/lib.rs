use std::io;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::RawGamepad;

pub struct VirtualGamepad {
    raw: RawGamepad,
    ty: GamepadType,
}

impl VirtualGamepad {
    pub fn new(ty: GamepadType) -> io::Result<Self> {
        let info = ty.info();
        Ok(Self {
            raw: RawGamepad::new(info.vendor_id, info.product_id, ty.name())?,
            ty,
        })
    }

    pub const fn ty(&self) -> GamepadType {
        self.ty
    }

    pub fn update(&mut self, update: GamepadUpdate) {
        match update {
            GamepadUpdate::Button { button, pressed } => self.raw.update_button(button, pressed),
            GamepadUpdate::Trigger { trigger, value } => self.raw.update_trigger(trigger, value),
            GamepadUpdate::Joystick { joystick, x, y } => self.raw.update_joystick(joystick, x, y),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadButton {
    /// Xbox360: Y
    /// DualShock4: Triangle
    North,

    /// Xbox360: A
    /// DualShock4: X
    South,

    /// Xbox360: B
    /// DualShock4: Circle
    East,

    /// Xbox360: X
    /// DualShock4: Square
    West,

    /// Directional Pad Up
    DPadUp,

    /// Directional Pad Down
    DPadDown,

    /// Directional Pad Left
    DPadLeft,

    /// Directional Pad Right
    DPadRight,

    /// Left stick click
    LeftThumb,

    /// Right stick click
    RightThumb,

    /// Menu button on the right
    /// Nintendo Switch: Plus
    Start,

    /// Menu button on the left.
    /// Nintendo Switch: Minus
    Select,

    /// Branded button, such as the big X in the middle of the Xbox360 Controller.
    Mode,

    /// Right bumper, not analog.
    RightBumper,

    /// Left bumper, not analog.
    LeftBumper,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Joystick {
    /// Left joystick, usually used for movement
    Left,

    /// Right joystick, usually used for camera rotation.
    Right,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Trigger {
    /// Left Analog Trigger
    Left,

    /// Right Analog Trigger
    Right,
}

/// A change to the state of a button, trigger, or joystick on a gamepad.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadUpdate {
    /// Buttons are simple, they are either on or off.
    Button {
        /// The button whose state changed.
        button: GamepadButton,

        /// Whether the button is pressed or released.
        pressed: bool,
    },

    /// Triggers are analog, so they have their own variant.
    Trigger {
        /// Left or Right trigger.
        trigger: Trigger,

        /// Activation value in the range [0.0,1.0]
        value: f32,
    },

    /// Joysticks have an X and a Y axis activation.
    Joystick {
        /// Left or right stick.
        joystick: Joystick,

        /// X axis activation value in the range [-1.0,1.0]
        x: f32,

        /// Y axis activation value in the range [-1.0,1.0]
        y: f32,
    },
}

/// This is used to tell the OS what kind of controller is connected.
/// If we didn't specify this correctly, most games wouldn't detect
/// the device.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamepadType {
    Xbox360,
    DualShock4,
}

impl GamepadType {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Xbox360 => "Xbox360",
            Self::DualShock4 => "DualShock4",
        }
    }

    pub const fn info(&self) -> GamepadInfo {
        // Use this: https://gist.github.com/nondebug/aec93dff7f0f1969f4cc2291b24a3171
        let (vendor, product) = match self {
            Self::Xbox360 => (0x045e, 0x028e),
            Self::DualShock4 => (0x54c, 0x5c4),
        };
        GamepadInfo {
            vendor_id: vendor,
            product_id: product,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadInfo {
    pub vendor_id: u16,
    pub product_id: u16,
}

fn dequantize(v: f32) -> i32 {
    (v * 32767.0) as i32
}
