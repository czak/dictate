//! Types text into the focused window through a [`VirtualKeyboard`].
//!
//! The virtual keyboard uploads a keymap for a fixed layout, so typing does not
//! depend on the seat's keyboard or on the compositor. For each character, the
//! matching key and modifiers come from [`Keymap`].

use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::keymap::Keymap;
use crate::wayland::{VirtualKeyboard, Wayland};

/// Pause after a keymap upload so the focused client can apply it. Without it
/// the client drops the first key.
const SETTLE: Duration = Duration::from_millis(10);

/// Flush and sync with the compositor this often while typing, so the socket
/// buffer does not fill up on long text.
const ROUNDTRIP_EVERY: usize = 64;

/// Characters that have no key, such as newlines, are never typed.
fn is_typable(c: &char) -> bool {
    !c.is_control()
}

/// Types text into the focused window using a fixed layout.
pub struct Typist {
    keyboard: VirtualKeyboard,
    keymap: Keymap,
}

impl Typist {
    /// Creates the virtual keyboard and uploads the keymap for `layout`, an XKB
    /// layout name such as `pl`.
    pub fn new(wl: &Wayland, layout: &str) -> Result<Self> {
        let keyboard = VirtualKeyboard::new(wl)?;
        let keymap = Keymap::from_layout(layout)?;
        keyboard.set_keymap(keymap.text())?;
        thread::sleep(SETTLE);
        Ok(Self { keyboard, keymap })
    }

    /// Types `text` into the focused window and returns how many characters had
    /// no key and were skipped. Blocks until the compositor has received the
    /// events.
    pub fn type_text(&mut self, text: &str) -> Result<usize> {
        let mut skipped = 0;
        let mut typed = 0;
        for c in text.chars().filter(is_typable) {
            let Some(stroke) = self.keymap.stroke(c) else {
                skipped += 1;
                continue;
            };
            // Set the modifiers only when needed, so plain keys send none.
            if stroke.mods != 0 {
                self.keyboard.modifiers(stroke.mods);
            }
            self.keyboard.tap(stroke.code);
            if stroke.mods != 0 {
                self.keyboard.modifiers(0);
            }
            typed += 1;
            if typed % ROUNDTRIP_EVERY == 0 {
                self.keyboard.sync()?;
            }
        }
        self.keyboard.sync()?;
        Ok(skipped)
    }
}

#[cfg(test)]
mod tests {
    /// Types into whatever window has keyboard focus. Never run it against a live
    /// session: it types into the focused window. Point `XDG_RUNTIME_DIR` and
    /// `WAYLAND_DISPLAY` at a private headless compositor with a text client.
    #[test]
    #[ignore = "needs a Wayland compositor with a focused window"]
    fn types_into_focused_window() {
        let text = std::env::var("DICTATE_TEST_TEXT")
            .unwrap_or_else(|_| "Zażółć gęślą jaźń 123".to_string());
        let wl = crate::wayland::Wayland::connect().unwrap();
        let mut typist = super::Typist::new(&wl, "pl").unwrap();
        // Two calls, as the server does, split on a character boundary.
        let mid = text
            .char_indices()
            .nth(text.chars().count() / 2)
            .map_or(text.len(), |(i, _)| i);
        let (first, second) = text.split_at(mid);
        assert_eq!(typist.type_text(first).unwrap(), 0);
        assert_eq!(typist.type_text(second).unwrap(), 0);
    }
}
