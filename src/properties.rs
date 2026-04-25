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
    /// Get the name of the gamepad as a string.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Xbox360 => "Xbox360",
            Self::DualShock4 => "DualShock4",
        }
    }

    /// Get the vendor/product ID of the gamepad.
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
