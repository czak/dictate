//! Builds an XKB keymap for a layout name such as `pl`, and finds, for each
//! character, the key to press and the modifiers to hold with it.
//!
//! The modifier masks come from the keymap itself, so they match whatever
//! modifier names the keymap uses (for example AltGr may be Mod3 or Mod5).

use std::collections::HashMap;

use anyhow::{Context, Result};
use xkbcommon::xkb;

/// How to type one character: an evdev key code and the modifier mask that
/// must be held while the key is pressed. A mask of 0 means no modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stroke {
    /// Evdev key code. This is the XKB keycode minus 8.
    pub code: u32,
    pub mods: u32,
}

/// A compiled keymap, its source text, and the strokes that produce each
/// character on it.
pub struct Keymap {
    text: String,
    strokes: HashMap<char, Stroke>,
}

/// Upper bound on the masks read per level. XKB levels rarely need more than a
/// few masks, so this is generous.
const MAX_MASKS: usize = 16;

impl Keymap {
    /// Builds the keymap for `layout`, an XKB layout name such as `pl`, from the
    /// system's xkb data. Fails if the layout is unknown.
    pub fn from_layout(layout: &str) -> Result<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "evdev",
            "pc105",
            layout,
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .with_context(|| format!("cannot build keymap for layout {layout:?}"))?;
        let text = keymap.get_as_string(xkb::KEYMAP_FORMAT_TEXT_V1);
        Ok(Self::index(&keymap, text))
    }

    /// Compiles the XKB source `text` and indexes every character it can
    /// produce.
    #[cfg(test)]
    pub fn parse(text: String) -> Result<Self> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_string(
            &context,
            text.clone(),
            xkb::KEYMAP_FORMAT_TEXT_V1,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .context("cannot compile keymap")?;
        Ok(Self::index(&keymap, text))
    }

    /// The keymap source this was built from.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The stroke that types `c`, or `None` if the keymap cannot produce it.
    pub fn stroke(&self, c: char) -> Option<Stroke> {
        self.strokes.get(&c).copied()
    }

    /// Indexes the strokes of a compiled `keymap` whose source is `text`.
    /// Where a character can be typed in more than one way, the stroke with the
    /// fewest modifiers wins.
    fn index(keymap: &xkb::Keymap, text: String) -> Self {
        let mut strokes: HashMap<char, Stroke> = HashMap::new();
        for raw in keymap.min_keycode().raw()..=keymap.max_keycode().raw() {
            let key = xkb::Keycode::new(raw);
            // Layout 0 is the one the virtual keyboard uses, since no group is set.
            if keymap.num_layouts_for_key(key) == 0 {
                continue;
            }
            for level in 0..keymap.num_levels_for_key(key, 0) {
                let mut masks = [0; MAX_MASKS];
                let count = keymap.key_get_mods_for_level(key, 0, level, &mut masks);
                let Some(&mods) = masks[..count.min(MAX_MASKS)]
                    .iter()
                    .min_by_key(|m| (m.count_ones(), **m))
                else {
                    continue;
                };
                let stroke = Stroke {
                    code: raw - 8,
                    mods,
                };
                for &sym in keymap.key_get_syms_by_level(key, 0, level) {
                    let Some(c) = char::from_u32(xkb::keysym_to_utf32(sym)) else {
                        continue;
                    };
                    if c.is_control() {
                        continue;
                    }
                    strokes
                        .entry(c)
                        .and_modify(|best| {
                            if stroke.mods.count_ones() < best.mods.count_ones() {
                                *best = stroke;
                            }
                        })
                        .or_insert(stroke);
                }
            }
        }
        Self { text, strokes }
    }
}

#[cfg(test)]
mod tests {
    use super::{Keymap, Stroke};

    /// Keys: `a` (evdev 30) with Shift for `A`; `l` (evdev 38) with AltGr for
    /// `ł`, where AltGr is Mod3 (mask 32), not the usual Mod5 (mask 128);
    /// space (evdev 57); Tab (evdev 15), which is a control character and must
    /// be ignored.
    const KEYMAP: &str = r#"xkb_keymap {
xkb_keycodes { minimum = 8; maximum = 70; <AC01> = 38; <AC09> = 46; <SPCE> = 65; <TAB> = 23; };
xkb_types { include "complete"
  type "ALTGR" {
    modifiers = Mod3;
    map[Mod3] = Level2;
    level_name[Level1] = "Base";
    level_name[Level2] = "AltGr";
  };
};
xkb_compatibility { include "complete" };
xkb_symbols {
  key <AC01> { type = "TWO_LEVEL", [ U0061, U0041 ] };
  key <AC09> { type = "ALTGR", [ U006C, U0142 ] };
  key <SPCE> { [ U0020 ] };
  key <TAB> { [ Tab ] };
};
};"#;

    fn parse() -> Keymap {
        Keymap::parse(KEYMAP.to_string()).unwrap()
    }

    #[test]
    fn plain_key_needs_no_modifiers() {
        assert_eq!(parse().stroke('a'), Some(Stroke { code: 30, mods: 0 }));
    }

    #[test]
    fn shifted_key_reports_the_keymap_mask() {
        let stroke = parse().stroke('A').unwrap();
        assert_eq!(stroke.code, 30);
        assert_eq!(stroke.mods, 1, "Shift is bit 0 in every keymap");
    }

    #[test]
    fn level_three_uses_the_keymap_own_modifier() {
        // Mod3 is mask 32. Mod5, the usual AltGr modifier, would be 128.
        let stroke = parse().stroke('ł').unwrap();
        assert_eq!(stroke.code, 38);
        assert_eq!(stroke.mods, 32);
    }

    #[test]
    fn space_is_typable() {
        assert_eq!(parse().stroke(' '), Some(Stroke { code: 57, mods: 0 }));
    }

    #[test]
    fn control_characters_and_unknown_characters_have_no_stroke() {
        let keymap = parse();
        assert_eq!(keymap.stroke('\t'), None);
        assert_eq!(keymap.stroke('ж'), None);
    }

    #[test]
    fn keeps_the_source_text() {
        assert_eq!(parse().text(), KEYMAP);
    }

    #[test]
    fn rejects_invalid_keymap() {
        assert!(Keymap::parse("not a keymap".to_string()).is_err());
    }

    #[test]
    fn polish_layout_types_polish_letters() {
        let keymap = Keymap::from_layout("pl").unwrap();
        for c in "aZ0 ąęłżŻ".chars() {
            assert!(keymap.stroke(c).is_some(), "missing {c:?}");
        }
        assert_eq!(keymap.stroke('a').unwrap().mods, 0);
    }

    #[test]
    fn unknown_layout_is_an_error() {
        assert!(Keymap::from_layout("no-such-layout").is_err());
    }
}
