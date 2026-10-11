//! Types text into the focused window through a [`VirtualKeyboard`].
//!
//! The keyboard is driven by a keymap that assigns one keycode per character,
//! so typing does not depend on the seat's layout. The keymap starts with a
//! fixed set of characters, which covers most dictation. Characters outside it
//! trigger a keymap upload for that utterance.

use std::thread;
use std::time::Duration;

use anyhow::Result;

use crate::wayland::{VirtualKeyboard, Wayland};

/// Pause after a keymap upload so the focused client can apply it. Without it
/// the client drops the first key.
const SETTLE: Duration = Duration::from_millis(10);

/// Flush and sync with the compositor this often while typing, so the socket
/// buffer does not fill up on long text.
const ROUNDTRIP_EVERY: usize = 64;

/// XKB keycodes span 8..=255, and evdev code `n` is sent as XKB keycode `n + 8`.
/// The keymap therefore holds at most 247 keys.
const MAX_KEYS: usize = 247;

/// Characters that the keymap always contains: printable ASCII, Polish letters,
/// and common punctuation.
fn base_layout() -> Vec<char> {
    let mut layout: Vec<char> = (0x20u8..=0x7e).map(char::from).collect();
    layout.extend("ĄĆĘŁŃÓŚŹŻąćęłńóśźż„”‘’–—…€".chars());
    layout
}

/// Returns the keymap to use for `wanted`. It is `current` unchanged when every
/// wanted character is already present. Otherwise it appends the missing ones,
/// or if that would overflow, it resets to the base layout plus as many missing
/// characters as fit.
pub fn plan_layout(current: &[char], wanted: &[char]) -> Vec<char> {
    let mut missing: Vec<char> = Vec::new();
    for &c in wanted {
        if !current.contains(&c) && !missing.contains(&c) {
            missing.push(c);
        }
    }
    if missing.is_empty() {
        return current.to_vec();
    }
    if current.len() + missing.len() <= MAX_KEYS {
        return current.iter().chain(&missing).copied().collect();
    }
    let mut layout = base_layout();
    for c in missing {
        if layout.len() < MAX_KEYS {
            layout.push(c);
        }
    }
    layout
}

/// Builds the XKB keymap text for `layout`. Key `i` (1-based) is evdev code `i`,
/// sent to the client as XKB keycode `i + 8`, and produces the `i`-th character.
pub fn keymap_text(layout: &[char]) -> String {
    let n = layout.len();
    let mut s = String::from("xkb_keymap {\nxkb_keycodes \"(unnamed)\" {\nminimum = 8;\n");
    s += &format!("maximum = {};\n", n + 8);
    for i in 1..=n {
        s += &format!("<K{i}> = {};\n", i + 8);
    }
    s += "};\n";
    s += "xkb_types \"(unnamed)\" { include \"complete\" };\n";
    s += "xkb_compatibility \"(unnamed)\" { include \"complete\" };\n";
    s += "xkb_symbols \"(unnamed)\" {\n";
    for (i, &c) in layout.iter().enumerate() {
        // `U<hex>` is the XKB name for a Unicode keysym.
        s += &format!("key <K{}> {{[ U{:04X} ]}};\n", i + 1, c as u32);
    }
    s += "};\n};\n";
    s
}

/// Keys that the keymap can produce. Control characters, such as newlines, are
/// skipped.
fn is_typable(c: &char) -> bool {
    !c.is_control()
}

/// Types text into the focused window using the current keymap.
pub struct Typist {
    keyboard: VirtualKeyboard,
    /// The keymap currently uploaded to the compositor.
    layout: Vec<char>,
}

impl Typist {
    /// Creates the virtual keyboard and uploads the base keymap.
    pub fn new(wl: &Wayland) -> Result<Self> {
        let mut typist = Self {
            keyboard: VirtualKeyboard::new(wl)?,
            layout: Vec::new(),
        };
        typist.upload(base_layout())?;
        Ok(typist)
    }

    /// Types `text` into the focused window and returns how many characters had
    /// no key and were skipped. Blocks until the compositor has received the
    /// events.
    pub fn type_text(&mut self, text: &str) -> Result<usize> {
        let wanted: Vec<char> = text.chars().filter(is_typable).collect();
        let layout = plan_layout(&self.layout, &wanted);
        if layout != self.layout {
            self.upload(layout)?;
        }

        let mut skipped = 0;
        let mut typed = 0;
        for c in text.chars().filter(is_typable) {
            let Some(index) = self.layout.iter().position(|&k| k == c) else {
                skipped += 1;
                continue;
            };
            self.keyboard.tap(index as u32 + 1);
            typed += 1;
            if typed % ROUNDTRIP_EVERY == 0 {
                self.keyboard.sync()?;
            }
        }
        self.keyboard.sync()?;
        Ok(skipped)
    }

    fn upload(&mut self, layout: Vec<char>) -> Result<()> {
        self.keyboard.set_keymap(&keymap_text(&layout))?;
        self.layout = layout;
        thread::sleep(SETTLE);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_KEYS, base_layout, keymap_text, plan_layout};

    fn chars(s: &str) -> Vec<char> {
        s.chars().collect()
    }

    #[test]
    fn base_layout_covers_ascii_and_polish() {
        let layout = base_layout();
        for c in "aZ0 ~ąęłżŻ„”".chars() {
            assert!(layout.contains(&c), "missing {c:?}");
        }
        assert!(layout.len() <= MAX_KEYS);
    }

    #[test]
    fn plan_keeps_layout_when_all_characters_are_present() {
        let current = base_layout();
        assert_eq!(plan_layout(&current, &chars("abc ą")), current);
    }

    #[test]
    fn plan_appends_missing_characters_once() {
        let current = base_layout();
        // `€` is already in the base layout, so only `ж` is missing.
        let planned = plan_layout(&current, &chars("ж€ж"));
        assert_eq!(planned.len(), current.len() + 1);
        assert_eq!(&planned[..current.len()], current.as_slice());
        assert_eq!(planned.last(), Some(&'ж'));
    }

    #[test]
    fn plan_resets_to_base_when_layout_would_overflow() {
        let current: Vec<char> = (0..MAX_KEYS as u32)
            .filter_map(|n| char::from_u32(0x4e00 + n))
            .collect();
        let planned = plan_layout(&current, &chars("ж"));
        assert!(planned.len() <= MAX_KEYS);
        assert!(planned.contains(&'ż'));
        assert_eq!(planned.last(), Some(&'ж'));
    }

    /// Types into whatever window has keyboard focus. Never run it against a live
    /// session: it types into the focused window. Point `XDG_RUNTIME_DIR` and
    /// `WAYLAND_DISPLAY` at a private headless compositor with a text client.
    #[test]
    #[ignore = "needs a Wayland compositor with a focused window"]
    fn types_into_focused_window() {
        let text = std::env::var("DICTATE_TEST_TEXT")
            .unwrap_or_else(|_| "Zażółć gęślą jaźń 123".to_string());
        let wl = crate::wayland::Wayland::connect().unwrap();
        let mut typist = super::Typist::new(&wl).unwrap();
        assert_eq!(typist.type_text(&text).unwrap(), 0);
    }

    #[test]
    fn keymap_names_keys_and_keycodes() {
        let text = keymap_text(&chars(" a"));
        assert!(text.contains("maximum = 10;"));
        assert!(text.contains("<K1> = 9;"));
        assert!(text.contains("<K2> = 10;"));
        assert!(text.contains("key <K1> {[ U0020 ]};"));
        assert!(text.contains("key <K2> {[ U0061 ]};"));
    }
}
