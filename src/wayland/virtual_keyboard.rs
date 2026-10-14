//! A `zwp_virtual_keyboard_v1` attached to the first seat. It sends key events
//! and keymaps to the focused window. It knows nothing about text or layouts.

use std::fs::File;
use std::io::Write;
use std::os::fd::AsFd;
use std::time::Instant;

use anyhow::Result;
use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::protocol::wl_keyboard;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

use super::Wayland;

pub struct VirtualKeyboard {
    wl: Wayland,
    keyboard: ZwpVirtualKeyboardV1,
    start: Instant,
}

impl VirtualKeyboard {
    /// Fails if the compositor lacks the virtual keyboard protocol or a seat.
    pub fn new(wl: &Wayland) -> Result<Self> {
        let seat = wl.seat()?;
        let manager = wl.virtual_keyboard_manager()?;
        let keyboard = manager.create_virtual_keyboard(&seat, &wl.qh(), ());
        Ok(Self {
            wl: wl.clone(),
            keyboard,
            start: Instant::now(),
        })
    }

    /// Replaces the keymap with the XKB source `xkb` and waits until the
    /// compositor has it. The client may still need a moment to apply it.
    pub fn set_keymap(&self, xkb: &str) -> Result<()> {
        let mut bytes = xkb.as_bytes().to_vec();
        bytes.push(0);
        let fd = memfd_create("dictate-keymap", MemfdFlags::CLOEXEC)?;
        let mut file = File::from(fd);
        file.write_all(&bytes)?;

        self.keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1.into(),
            file.as_fd(),
            bytes.len() as u32,
        );
        // The file must stay open until the compositor has received the fd.
        self.wl.roundtrip()?;
        drop(file);
        Ok(())
    }

    /// Presses and releases the key `code`, an evdev code under the current
    /// keymap. The events are queued until the next [`sync`](Self::sync).
    pub fn tap(&self, code: u32) {
        let time = self.start.elapsed().as_millis() as u32;
        self.keyboard
            .key(time, code, wl_keyboard::KeyState::Pressed.into());
        self.keyboard
            .key(time, code, wl_keyboard::KeyState::Released.into());
    }

    /// Sets the modifiers held down for the following key events. `depressed`
    /// is a bit mask of the keymap's modifiers, such as Shift. It replaces any
    /// earlier value. Pass 0 to release them all.
    pub fn modifiers(&self, depressed: u32) {
        self.keyboard.modifiers(depressed, 0, 0, 0);
    }

    /// Sends queued events and waits until the compositor has handled them.
    pub fn sync(&self) -> Result<()> {
        self.wl.roundtrip()
    }
}
