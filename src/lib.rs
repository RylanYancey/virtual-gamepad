#[cfg(target_os = "linux")]
mod linux;
use std::io;

#[cfg(target_os = "linux")]
use linux::RawGamepad;

pub use crate::{
    properties::{GamepadInfo, GamepadType},
    state::{Buttons, GamepadState},
};

pub mod properties;
pub mod state;

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
}
