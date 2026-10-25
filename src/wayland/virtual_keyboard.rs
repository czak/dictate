//! A `zwp_virtual_keyboard_v1` attached to the first seat. It sends key events
//! and keymaps to the focused window. It knows nothing about text or layouts.

use std::fs::File;
use std::io::Write;
use std::os::fd::AsFd;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::protocol::wl_keyboard;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;

use super::Wayland;

/// Pause after a keymap upload so the focused client can apply it. Without it
/// the client drops the first key.
const SETTLE: Duration = Duration::from_millis(10);

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

    /// Replaces the keymap with the XKB source `xkb`, waits until the compositor
    /// has it, then pauses so the focused client can apply it.
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
        thread::sleep(SETTLE);
        Ok(())
    }

    /// Presses and releases `code`, an evdev code under the current keymap,
    /// with the modifier mask `mods` held down. The events are queued until the
    /// next [`sync`](Self::sync).
    pub fn tap(&self, code: u32, mods: u32) {
        let time = self.start.elapsed().as_millis() as u32;
        if mods != 0 {
            self.keyboard.modifiers(mods, 0, 0, 0);
        }
        self.keyboard
            .key(time, code, wl_keyboard::KeyState::Pressed.into());
        self.keyboard
            .key(time, code, wl_keyboard::KeyState::Released.into());
        if mods != 0 {
            self.keyboard.modifiers(0, 0, 0, 0);
        }
    }

    /// Sends queued events and waits until the compositor has handled them.
    pub fn sync(&self) -> Result<()> {
        self.wl.roundtrip()
    }
}
