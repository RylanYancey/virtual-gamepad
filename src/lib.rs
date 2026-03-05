use std::io;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::RawGamepad;

/// A type that behaves like a gamepad, without the need for a physical device.
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
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum GamepadButton {
    /// Xbox360: Y
    /// DualShock4: Triangle
    North = 0,

    /// Xbox360: A
    /// DualShock4: X
    South = 1,

    /// Xbox360: B
    /// DualShock4: Circle
    East = 2,

    /// Xbox360: X
    /// DualShock4: Square
    West = 3,

    /// Directional Pad Up
    DPadUp = 4,

    /// Directional Pad Down
    DPadDown = 5,

    /// Directional Pad Left
    DPadLeft = 6,

    /// Directional Pad Right
    DPadRight = 7,

    /// Left stick click
    LeftThumb = 8,

    /// Right stick click
    RightThumb = 9,

    /// Menu button on the right
    /// Nintendo Switch: Plus
    Start = 10,

    /// Menu button on the left.
    /// Nintendo Switch: Minus
    Select = 11,

    /// Branded button, such as the big X in the middle of the Xbox360 Controller.
    Mode = 12,

    /// Right bumper, not analog.
    RightBumper = 13,

    /// Left bumper, not analog.
    LeftBumper = 14,
}

impl GamepadButton {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::North,
            1 => Self::South,
            2 => Self::East,
            3 => Self::West,
            4 => Self::DPadUp,
            5 => Self::DPadDown,
            6 => Self::DPadLeft,
            7 => Self::DPadRight,
            8 => Self::LeftThumb,
            9 => Self::RightThumb,
            10 => Self::Start,
            11 => Self::Select,
            12 => Self::Mode,
            13 => Self::RightBumper,
            14 => Self::LeftBumper,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum Joystick {
    /// Left joystick, usually used for movement
    Left = 0,

    /// Right joystick, usually used for camera rotation.
    Right = 1,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum Trigger {
    /// Left Analog Trigger
    Left = 0,

    /// Right Analog Trigger
    Right = 1,
}

/// A change to the state of a button, trigger, or joystick on a gamepad.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
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
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum GamepadType {
    Xbox360 = 0,
    DualShock4 = 1,
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
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub struct GamepadInfo {
    pub vendor_id: u16,
    pub product_id: u16,
}

/// Convert an f32 in the range [-1.0,1.0] to an i32 in the range [-32767,32768]
pub fn quantize(v: f32) -> i32 {
    (v * 32767.0) as i32
}

/// Convert an i32 in the range [-32767,32768] to an f32 in the range [-1.0,1.0]
pub fn dequantize(v: i32) -> f32 {
    v as f32 / 32767.0
}
