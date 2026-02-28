use crate::config::save_config;
use crate::controllers::Controller;
use crate::{AdventuriaApp, HotkeysConfigMessage, Message};
use global_hotkey::hotkey::HotKey;
use iced::Task;

pub struct HotkeysController {}

impl Controller for HotkeysController {
    fn update(state: &mut AdventuriaApp, message: Message) -> Task<Message> {
        match message {
            Message::HotkeysConfig(config_message) => match config_message {
                HotkeysConfigMessage::StartModifierChanged(modifier) => {
                    state.config.start_modifier = modifier;
                    HotkeysController::update_hotkeys(state);
                    Task::none()
                }
                HotkeysConfigMessage::StartKeyChanged(key) => {
                    state.config.start_key = key;
                    HotkeysController::update_hotkeys(state);
                    Task::none()
                }
                HotkeysConfigMessage::StopModifierChanged(modifier) => {
                    state.config.stop_modifier = modifier;
                    HotkeysController::update_hotkeys(state);
                    Task::none()
                }
                HotkeysConfigMessage::StopKeyChanged(key) => {
                    state.config.stop_key = key;
                    HotkeysController::update_hotkeys(state);
                    Task::none()
                }
            },
            _ => Task::none(),
        }
    }
}

impl HotkeysController {
    fn update_hotkeys(state: &mut AdventuriaApp) {
        let _ = state.hotkey_manager.unregister_all(&state.current_hotkeys);

        let hotkey_start = HotKey::new(
            state.config.start_modifier.to_global_modifiers(),
            state.config.start_key.into(),
        );
        let hotkey_stop = HotKey::new(
            state.config.stop_modifier.to_global_modifiers(),
            state.config.stop_key.into(),
        );

        state.hotkey_start_id = hotkey_start.id();
        state.hotkey_stop_id = hotkey_stop.id();

        let _ = state.hotkey_manager.register(hotkey_start);
        let _ = state.hotkey_manager.register(hotkey_stop);

        state.current_hotkeys = vec![hotkey_start, hotkey_stop];

        save_config(&state.config);
    }
}
