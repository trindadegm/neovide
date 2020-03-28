mod qwerty;

use log::{error, trace};
use skulpin::sdl2::keyboard::{Keycode, Mod};

use crate::settings::{FromValue, Value, SETTINGS};

use qwerty::*;

pub fn unsupported_key<R>(keycode: Keycode) -> Option<R> {
    trace!("Unsupported key: {:?}", keycode);
    None
}
#[derive(Clone)]
pub enum KeyboardLayout {
    Qwerty,
}

impl FromValue for KeyboardLayout {
    fn from_value(&mut self, value: Value) {
        match value.as_str() {
            Some("qwerty") => *self = KeyboardLayout::Qwerty,
            _ => error!(
                "keyboard_layout setting expected a known keyboard layout name, but received: {}",
                value
            ),
        }
    }
}

impl From<KeyboardLayout> for Value {
    fn from(layout: KeyboardLayout) -> Self {
        match layout {
            KeyboardLayout::Qwerty => "qwerty".into(),
        }
    }
}

#[derive(Clone)]
struct KeyboardSettings {
    layout: KeyboardLayout,
}

pub fn initialize_settings() {
    SETTINGS.set(&KeyboardSettings {
        layout: KeyboardLayout::Qwerty,
    });

    register_nvim_setting!("keyboard_layout", KeyboardSettings::layout);
}

fn append_modifiers(
    keycode_text: &str,
    special: bool,
    shift: bool,
    ctrl: bool,
    alt: bool,
    gui: bool,
) -> String {
    let mut result = keycode_text.to_string();
    let mut special = special;

    if result == "<" {
        result = "lt".to_string();
        special = true;
    }

    if shift {
        special = true;
        result = format!("S-{}", result);
    }
    if ctrl {
        special = true;
        result = format!("C-{}", result);
    }
    if alt {
        special = true;
        result = format!("M-{}", result);
    }
    if cfg!(not(target_os = "windows")) && gui {
        special = true;
        result = format!("D-{}", result);
    }

    if special {
        result = format!("<{}>", result);
    }

    result
}

pub fn produce_neovim_keybinding_string(
    keycode: Option<Keycode>,
    keytext: Option<String>,
    modifiers: Mod,
) -> Option<String> {
    let shift = modifiers.contains(Mod::LSHIFTMOD) || modifiers.contains(Mod::RSHIFTMOD);
    let ctrl = modifiers.contains(Mod::LCTRLMOD) || modifiers.contains(Mod::RCTRLMOD);
    // Right Alt (RALTMOD) is not considered because some keyboards replace it with AltGr,
    // (Alt Group) which functions in a similar fashion to Shift. It allows a different character to
    // be entered while it is pressed. When Right Alt was considered here, it would be impossible to
    // enter such characters.
    let alt = modifiers.contains(Mod::LALTMOD);
    let gui = modifiers.contains(Mod::LGUIMOD) || modifiers.contains(Mod::RGUIMOD);
    if let Some(text) = keytext {
        Some(append_modifiers(&text, false, false, ctrl, alt, gui))
    } else if let Some(keycode) = keycode {
        (match SETTINGS.get::<KeyboardSettings>().layout {
            KeyboardLayout::Qwerty => handle_qwerty_layout(keycode, shift, ctrl, alt),
        })
        .map(|(transformed_text, special, shift, ctrl, alt)| {
            append_modifiers(transformed_text, special, shift, ctrl, alt, gui)
        })
    } else {
        None
    }
}
