#[cfg(target_os = "linux")]
mod linux;
use std::io;

#[cfg(target_os = "linux")]
use linux::RawGamepad;

#[cfg(feature = "rumble")]
pub use crate::rumble::RumbleState;
pub use crate::{
    properties::{GamepadInfo, GamepadType},
    state::{Buttons, GamepadState},
};

mod properties;
#[cfg(feature = "rumble")]
mod rumble;
mod state;

/// A virtual gamepad, with which input events can be emitted
/// as if an actual gamepad was connected.
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

    pub fn update(&mut self, state: GamepadState) {
        self.raw.update(state);
    }

    #[cfg(feature = "rumble")]
    pub fn rumble(&mut self) -> RumbleState {
        self.raw.rumble()
    }
}
