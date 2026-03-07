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
        self.raw.update(update.button, update.values);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    /// Left Joystick X/Y
    LeftStick = 15,

    /// Right Joystick X/Y
    RightStick = 16,

    /// Left analog trigger.
    LeftTrigger = 17,

    /// Right analog trigger.
    RightTrigger = 18,
}

impl GamepadButton {
    pub const fn from_u8(v: u8) -> Option<Self> {
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
            15 => Self::LeftStick,
            16 => Self::RightStick,
            17 => Self::LeftTrigger,
            18 => Self::RightTrigger,
            _ => return None,
        })
    }

    pub const fn is_joystick(self) -> bool {
        matches!(self, Self::LeftStick | Self::RightStick)
    }

    pub const fn is_trigger(self) -> bool {
        matches!(self, Self::LeftTrigger | Self::RightTrigger)
    }
}

/// A change to the state of a button, trigger, or joystick on a gamepad.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GamepadUpdate {
    pub button: GamepadButton,
    pub values: [f32; 2],
}

impl GamepadUpdate {
    pub fn to_bytes(&self) -> [u8; 5] {
        let btn = self.button as u8;
        let [v00, v01] = quantize(self.values[0]).to_le_bytes();
        let [v10, v11] = quantize(self.values[1]).to_le_bytes();
        [btn, v00, v01, v10, v11]
    }

    pub fn from_bytes(&self, bytes: &[u8]) -> Option<Self> {
        // Convert to array of 5 bytes.
        let [btn, v00, v01, v10, v11] = bytes.try_into().ok()?;
        // Byte 0 encodes button type
        let button = GamepadButton::from_u8(btn)?;
        Some(Self {
            button,
            values: [
                // bytes 1 and 2 encode the X axis (or pressure vaule)
                dequantize(i16::from_le_bytes([v00, v01])).clamp(-1.0, 1.0),
                // bytes 3 and 4 encode the Y axis (or ignored)
                dequantize(i16::from_le_bytes([v10, v11])).clamp(-1.0, 1.0),
            ],
        })
    }
}

/// This is used to tell the OS what kind of controller is connected.
/// If we didn't specify this correctly, most games wouldn't detect
/// the device.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
pub struct GamepadInfo {
    pub vendor_id: u16,
    pub product_id: u16,
}

/// Convert an f32 in the range [-1.0,1.0] to an i16 in the range [-32767,32768]
pub fn quantize(v: f32) -> i16 {
    (v * 32767.0) as i16
}

/// Convert an i16 in the range [-32767,32768] to an f32 in the range [-1.0,1.0]
pub fn dequantize(v: i16) -> f32 {
    v as f32 / 32767.0
}
