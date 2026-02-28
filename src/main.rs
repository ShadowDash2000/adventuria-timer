#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod controllers;
mod hotkeys;
mod log;
mod views;

use crate::log::setup_panic_hook;
use config::{load_config, Config};
use global_hotkey::{
    hotkey::{Code, HotKey}, GlobalHotKeyEvent,
    GlobalHotKeyManager,
};
use hotkeys::HotkeyModifier;

use crate::controllers::{AuthResponse, Controller, TimerState};
use crate::views::View;
use iced::{Element, Task};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem}, TrayIcon, TrayIconBuilder,
    TrayIconEvent,
};

struct AdventuriaApp {
    domain: String,
    identity: String,
    password: String,
    status_message: String,
    hotkey_manager: GlobalHotKeyManager,
    hotkey_start_id: u32,
    hotkey_stop_id: u32,
    config: Config,
    timer: TimerState,
    current_hotkeys: Vec<HotKey>,
    _tray_icon: TrayIcon,
    show_item_id: tray_icon::menu::MenuId,
    quit_item_id: tray_icon::menu::MenuId,
    window_id: Option<iced::window::Id>,
}

#[derive(Debug, Clone)]
enum Message {
    DomainChanged(String),
    IdentityChanged(String),
    PasswordChanged(String),
    LoginPressed,
    LoginFinished(Result<AuthResponse, String>),
    HotKeyTriggered(u32),
    TimerRequestFinished(String, Result<(), String>),
    TrayEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    MinimizeToTray,
    WindowOpened(iced::window::Id),
    WindowFocused(),
    TimerStatusFinished(Result<bool, String>),
    HotkeysConfig(HotkeysConfigMessage),
}

#[derive(Debug, Clone)]
enum HotkeysConfigMessage {
    StartModifierChanged(HotkeyModifier),
    StartKeyChanged(Code),
    StopModifierChanged(HotkeyModifier),
    StopKeyChanged(Code),
}

impl Default for AdventuriaApp {
    fn default() -> Self {
        let (app, _) = Self::new();
        app
    }
}

impl AdventuriaApp {
    fn new() -> (Self, Task<Message>) {
        let mut status_message = "Ready".to_string();
        let config = load_config();
        let hotkey_manager = GlobalHotKeyManager::new().unwrap();

        let hotkey_start = HotKey::new(
            config.start_modifier.to_global_modifiers(),
            config.start_key.into(),
        );
        let hotkey_stop = HotKey::new(
            config.stop_modifier.to_global_modifiers(),
            config.stop_key.into(),
        );

        let hotkey_start_id = hotkey_start.id();
        let hotkey_stop_id = hotkey_stop.id();

        let mut hotkey_errors: Vec<String> = vec![];
        hotkey_manager.register(hotkey_start).unwrap_or_else(|err| {
            hotkey_errors.push(format!("Failed to register start hotkey: {err}"));
        });
        hotkey_manager.register(hotkey_stop).unwrap_or_else(|err| {
            hotkey_errors.push(format!("Failed to register stop hotkey: {err}"));
        });

        if !hotkey_errors.is_empty() {
            status_message = format!("Hotkey registration failed: {}", hotkey_errors.join(", "));
        }

        let tray_menu = Menu::new();
        let show_item = MenuItem::new("Show", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let show_item_id = show_item.id().clone();
        let quit_item_id = quit_item.id().clone();
        tray_menu.append(&show_item).unwrap();
        tray_menu.append(&quit_item).unwrap();

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Adventuria Timer")
            .with_icon(load_icon())
            .build()
            .unwrap();

        (
            Self {
                domain: "https://adventuria-api.tw1.su".to_string(),
                identity: "".to_string(),
                password: "".to_string(),
                status_message,
                hotkey_manager,
                hotkey_start_id,
                hotkey_stop_id,
                config,
                timer: TimerState { is_active: false },
                current_hotkeys: vec![hotkey_start, hotkey_stop],
                _tray_icon: tray_icon,
                show_item_id,
                quit_item_id,
                window_id: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::batch(vec![
            controllers::MainController::update(self, message.clone()),
            controllers::HotkeysController::update(self, message.clone()),
            controllers::TimerController::update(self, message.clone()),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        views::MainView::view(self)
    }
}

fn hotkey_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run(|| {
        iced::futures::stream::unfold((), |_| async move {
            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    return Some((Message::HotKeyTriggered(event.id), ()));
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        })
    })
}

fn tray_subscription() -> iced::Subscription<Message> {
    iced::Subscription::run(|| {
        let tray_receiver = TrayIconEvent::receiver();
        let menu_receiver = MenuEvent::receiver();

        iced::futures::stream::select(
            iced::futures::stream::unfold(tray_receiver, |receiver| async move {
                loop {
                    if let Ok(event) = receiver.try_recv() {
                        return Some((Message::TrayEvent(event), receiver));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }),
            iced::futures::stream::unfold(menu_receiver, |receiver| async move {
                loop {
                    if let Ok(event) = receiver.try_recv() {
                        return Some((Message::MenuEvent(event), receiver));
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }),
        )
    })
}

fn load_icon() -> tray_icon::Icon {
    let rgba = vec![255u8; 32 * 32 * 4];
    tray_icon::Icon::from_rgba(rgba, 32, 32).expect("Failed to create icon")
}

fn main() -> iced::Result {
    setup_panic_hook();

    iced::application(
        AdventuriaApp::default,
        AdventuriaApp::update,
        AdventuriaApp::view,
    )
    .window(iced::window::Settings {
        size: iced::Size::new(400.0, 450.0),
        ..Default::default()
    })
    .subscription(|_| {
        iced::Subscription::batch(vec![
            hotkey_subscription(),
            tray_subscription(),
            iced::window::events().map(|(id, event)| match event {
                iced::window::Event::Opened { .. } => Message::WindowOpened(id),
                iced::window::Event::Focused { .. } => Message::WindowFocused(),
                _ => Message::WindowOpened(id),
            }),
        ])
    })
    .run()
}
