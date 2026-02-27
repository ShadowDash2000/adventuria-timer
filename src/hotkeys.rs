use global_hotkey::hotkey::{Code, Modifiers as GlobalModifiers};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HotkeyModifier {
    #[default]
    Alt,
    Ctrl,
    Shift,
    None,
}

impl HotkeyModifier {
    pub const ALL: [HotkeyModifier; 4] = [
        HotkeyModifier::Alt,
        HotkeyModifier::Ctrl,
        HotkeyModifier::Shift,
        HotkeyModifier::None,
    ];

    pub fn to_global_modifiers(self) -> Option<GlobalModifiers> {
        match self {
            HotkeyModifier::Alt => Some(GlobalModifiers::ALT),
            HotkeyModifier::Ctrl => Some(GlobalModifiers::CONTROL),
            HotkeyModifier::Shift => Some(GlobalModifiers::SHIFT),
            HotkeyModifier::None => None,
        }
    }
}

impl std::fmt::Display for HotkeyModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotkeyModifier::Alt => write!(f, "Alt"),
            HotkeyModifier::Ctrl => write!(f, "Ctrl"),
            HotkeyModifier::Shift => write!(f, "Shift"),
            HotkeyModifier::None => write!(f, "None"),
        }
    }
}

pub const ALL_KEYS: [Code; 38] = [
    Code::KeyA,
    Code::KeyB,
    Code::KeyC,
    Code::KeyD,
    Code::KeyE,
    Code::KeyF,
    Code::KeyG,
    Code::KeyH,
    Code::KeyI,
    Code::KeyJ,
    Code::KeyK,
    Code::KeyL,
    Code::KeyM,
    Code::KeyN,
    Code::KeyO,
    Code::KeyP,
    Code::KeyQ,
    Code::KeyR,
    Code::KeyS,
    Code::KeyT,
    Code::KeyU,
    Code::KeyV,
    Code::KeyW,
    Code::KeyX,
    Code::KeyY,
    Code::KeyZ,
    Code::F1,
    Code::F2,
    Code::F3,
    Code::F4,
    Code::F5,
    Code::F6,
    Code::F7,
    Code::F8,
    Code::F9,
    Code::F10,
    Code::F11,
    Code::F12,
];
