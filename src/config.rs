use crate::hotkeys::HotkeyModifier;
use global_hotkey::hotkey::Code;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub token: Option<String>,
    pub user_id: Option<String>,
    #[serde(default = "default_start_modifier")]
    pub start_modifier: HotkeyModifier,
    #[serde(default = "default_start_key")]
    pub start_key: Code,
    #[serde(default = "default_stop_modifier")]
    pub stop_modifier: HotkeyModifier,
    #[serde(default = "default_stop_key")]
    pub stop_key: Code,
}

fn default_start_modifier() -> HotkeyModifier {
    HotkeyModifier::Alt
}
fn default_start_key() -> Code {
    Code::F8
}
fn default_stop_modifier() -> HotkeyModifier {
    HotkeyModifier::Alt
}
fn default_stop_key() -> Code {
    Code::F9
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token: None,
            user_id: None,
            start_modifier: default_start_modifier(),
            start_key: default_start_key(),
            stop_modifier: default_stop_modifier(),
            stop_key: default_stop_key(),
        }
    }
}

pub fn get_config_path() -> PathBuf {
    std::env::current_exe()
        .map(|p| p.parent().unwrap_or(&p).join("config.json"))
        .unwrap_or_else(|_| PathBuf::from("config.json"))
}

pub fn load_config() -> Config {
    let path = get_config_path();
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) {
    let path = get_config_path();
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, content);
    }
}
