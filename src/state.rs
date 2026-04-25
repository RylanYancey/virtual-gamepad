#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(from = "[u8; 14]", into = "[u8; 14]"))]
pub struct GamepadState {
    /// Bitmask of button states.
    pub buttons: Buttons,

    /// X left stick value.
    /// Quantized f32 in the range [-1.0,1.0]
    pub stick_left_x: i16,

    /// Y left stick value.
    /// Quantized f32 in the range [-1.0,1.0]
    pub stick_left_y: i16,

    /// X right stick value.
    /// Quantized f32 in the range [-1.0, 1.0]
    pub stick_right_x: i16,

    /// Y right stick value.
    /// Quantized f32 in the range [-1.0, 1.0]
    pub stick_right_y: i16,

    /// Left trigger value.
    /// Quantized f32 in the range [0.0,1.0]
    pub trigger_left: u8,

    /// Right trigger value.
    /// Quantized f32 in the range [0.0,1.0]
    pub trigger_right: u8,

    /// analog X directional pad value.
    /// Quantized f32 in the range [-1.0, 1.0].
    pub dpad_x: i8,

    /// Analog Y directional pad value.
    /// Quantized f32 in the range [-1.0, 1.0]
    pub dpad_y: i8,
}

impl GamepadState {
    /// Encodes gamepad state as array of 14 bytes for transmission over the network.
    /// Uses little endian byte ordering.
    /// Fields are encoded in the same order as they appear in the struct.
    pub const fn encode(self) -> [u8; 14] {
        let mask = self.buttons.bits().to_le_bytes();
        let slx = self.stick_left_x.to_le_bytes();
        let sly = self.stick_left_y.to_le_bytes();
        let srx = self.stick_right_x.to_le_bytes();
        let sry = self.stick_right_y.to_le_bytes();
        [
            mask[0],
            mask[1],
            slx[0],
            slx[1],
            sly[0],
            sly[1],
            srx[0],
            srx[1],
            sry[0],
            sry[1],
            self.trigger_left,
            self.trigger_right,
            self.dpad_x.to_le_bytes()[0],
            self.dpad_y.to_le_bytes()[0],
        ]
    }

    /// Decode gamepad state from array of 14 bytes.
    /// This cannot fail.
    pub const fn decode(data: [u8; 14]) -> Self {
        Self {
            buttons: Buttons::from_bits_truncate(u16::from_le_bytes([data[0], data[1]])),
            stick_left_x: i16::from_le_bytes([data[2], data[3]]),
            stick_left_y: i16::from_le_bytes([data[4], data[5]]),
            stick_right_x: i16::from_le_bytes([data[6], data[7]]),
            stick_right_y: i16::from_le_bytes([data[8], data[9]]),
            trigger_left: data[10],
            trigger_right: data[11],
            dpad_x: i8::from_le_bytes([data[12]]),
            dpad_y: i8::from_le_bytes([data[13]]),
        }
    }

    /// Quantize and set left stick values.
    pub const fn set_left_stick(&mut self, x: f32, y: f32) {
        self.stick_left_x = quantize_i16(x);
        self.stick_left_y = quantize_i16(y);
    }

    /// Quantize and set right stick values.
    pub const fn set_right_stick(&mut self, x: f32, y: f32) {
        self.stick_right_x = quantize_i16(x);
        self.stick_right_y = quantize_i16(y);
    }

    /// Quantize and set left trigger value.
    pub const fn set_trigger_left(&mut self, v: f32) {
        self.trigger_left = quantize_u8(v);
    }

    /// Quantize and set right trigger value.
    pub const fn set_trigger_right(&mut self, v: f32) {
        self.trigger_right = quantize_u8(v);
    }

    /// Quantize and set [X, Y] directional pad values.
    pub const fn set_dpad(&mut self, x: f32, y: f32) {
        self.dpad_x = quantize_i8(x);
        self.dpad_y = quantize_i8(y);
    }

    /// Dequentize [X, Y] left-stick values.
    pub const fn get_left_stick(&self) -> [f32; 2] {
        [
            dequantize_i16(self.stick_left_x),
            dequantize_i16(self.stick_left_y),
        ]
    }

    /// Dequantize [X, Y] right-stick values.
    pub const fn get_right_stick(&self) -> [f32; 2] {
        [
            dequantize_i16(self.stick_right_x),
            dequantize_i16(self.stick_right_y),
        ]
    }

    /// Dequantize left analog trigger.
    pub const fn get_trigger_left(&self) -> f32 {
        dequantize_u8(self.trigger_left)
    }

    /// Dequantize right analog trigger.
    pub const fn get_trigger_right(&self) -> f32 {
        dequantize_u8(self.trigger_right)
    }

    /// Dequantize directional pad values.
    pub const fn get_dpad(&self) -> [f32; 2] {
        [dequantize_i8(self.dpad_x), dequantize_i8(self.dpad_y)]
    }
}

impl Default for GamepadState {
    fn default() -> Self {
        Self {
            buttons: Buttons::empty(),
            stick_left_x: 0,
            stick_left_y: 0,
            stick_right_x: 0,
            stick_right_y: 0,
            trigger_left: 0,
            trigger_right: 0,
            dpad_x: 0,
            dpad_y: 0,
        }
    }
}

impl Into<[u8; 14]> for GamepadState {
    fn into(self) -> [u8; 14] {
        self.encode()
    }
}

impl From<[u8; 14]> for GamepadState {
    fn from(value: [u8; 14]) -> Self {
        Self::decode(value)
    }
}

bitflags::bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct Buttons: u16 {
        /// XBox360: Y,
        /// DualShock4: Triangle
        const NORTH = 1<<0;

        /// XBox360: A
        /// DualShock4: X
        const SOUTH = 1<<1;

        /// XBox360: B
        /// DualShock4: Circle
        const EAST = 1<<2;

        /// Xbox360: X
        /// DualShock4: Square
        const WEST = 1<<3;

        /// Left non-analog bumper.
        const BUMPER_LEFT = 1<<4;

        // Right non-analog bumper.
        const BUMPER_RIGHT = 1<<5;

        /// Left thumb stick click.
        const THUMB_LEFT = 1<<6;

        /// Right thumb stick click.
        const THUMB_RIGHT = 1<<7;

        /// Menu button right
        /// Nintendo Switch: Plus
        const START = 1<<8;

        /// Menu button left.
        /// Nintendo Switch: Minus
        const SELECT = 1<<9;

        /// Branded button.
        /// Xbox360: Big X button.
        /// Nintendo Switch: Home button
        const MODE = 1<<10;
    }
}

impl Buttons {
    /// All buttons that are in self and not prev.
    pub const fn just_pressed(self, prev: Self) -> Self {
        self.difference(prev)
    }

    /// All buttons that are in prev and not self.
    pub const fn just_released(self, prev: Self) -> Self {
        prev.difference(self)
    }
}

/// Compresses an f32 in the range [-1.0, 1.0] to an i16.
pub const fn quantize_i16(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * 32767.0) as i16
}

/// Decompresses an i16 to an f32 in the range [-1.0, 1.0].
pub const fn dequantize_i16(v: i16) -> f32 {
    v as f32 * 32767.0
}

/// Compresses an f32 in the range [0.0, 1.0] to a u8.
pub const fn quantize_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0) as u8
}

/// Decompresses a u8 to an f32 in the range [0.0, 1.0].
pub const fn dequantize_u8(v: u8) -> f32 {
    v as f32 * 255.0
}

/// Compresses an f32 in the range [-1.0, 1.0] to an i8.
pub const fn quantize_i8(v: f32) -> i8 {
    (v.clamp(-1.0, 1.0) * 127.0) as i8
}

/// Decompresses an i8 to an f32 in the range [-1.0, 1.0].
pub const fn dequantize_i8(v: i8) -> f32 {
    v as f32 * 127.0
}
