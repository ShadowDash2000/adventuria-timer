#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod config;
mod hotkeys;
mod log;

use crate::log::setup_panic_hook;
use config::{load_config, save_config, Config};
use global_hotkey::{
    hotkey::{Code, HotKey}, GlobalHotKeyEvent,
    GlobalHotKeyManager,
};
use hotkeys::{HotkeyModifier, ALL_KEYS};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length, Task};
use serde::{Deserialize, Serialize};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem}, TrayIcon, TrayIconBuilder,
    TrayIconEvent,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AuthResponse {
    token: String,
}

struct AdventuriaApp {
    domain: String,
    identity: String,
    password: String,
    status_message: String,
    hotkey_manager: GlobalHotKeyManager,
    hotkey_start_id: u32,
    hotkey_stop_id: u32,
    config: Config,
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
    LoginFinished(Result<String, String>),
    HotKeyTriggered(u32),
    TimerRequestFinished(String, Result<(), String>),
    TrayEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    MinimizeToTray,
    WindowOpened(iced::window::Id),
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

        hotkey_manager.register(hotkey_start).unwrap();
        hotkey_manager.register(hotkey_stop).unwrap();

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
                status_message: "Ready".to_string(),
                hotkey_manager,
                hotkey_start_id,
                hotkey_stop_id,
                config,
                current_hotkeys: vec![hotkey_start, hotkey_stop],
                _tray_icon: tray_icon,
                show_item_id,
                quit_item_id,
                window_id: None,
            },
            Task::none(),
        )
    }

    fn update_hotkeys(&mut self) {
        let _ = self.hotkey_manager.unregister_all(&self.current_hotkeys);

        let hotkey_start = HotKey::new(
            self.config.start_modifier.to_global_modifiers(),
            self.config.start_key.into(),
        );
        let hotkey_stop = HotKey::new(
            self.config.stop_modifier.to_global_modifiers(),
            self.config.stop_key.into(),
        );

        self.hotkey_start_id = hotkey_start.id();
        self.hotkey_stop_id = hotkey_stop.id();

        let _ = self.hotkey_manager.register(hotkey_start);
        let _ = self.hotkey_manager.register(hotkey_stop);

        self.current_hotkeys = vec![hotkey_start, hotkey_stop];

        save_config(&self.config);
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::DomainChanged(domain) => {
                self.domain = domain;
                Task::none()
            }
            Message::IdentityChanged(identity) => {
                self.identity = identity;
                Task::none()
            }
            Message::PasswordChanged(password) => {
                self.password = password;
                Task::none()
            }
            Message::LoginPressed => {
                self.status_message = "Logging in...".to_string();
                let domain = self.domain.clone();
                let identity = self.identity.clone();
                let password = self.password.clone();

                Task::perform(
                    async move {
                        let client = reqwest::Client::new();
                        let url = format!("{}/api/collections/users/auth-with-password", domain);
                        let body = serde_json::json!({
                            "identity": identity,
                            "password": password,
                        });

                        match client.post(&url).json(&body).send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    resp.json::<AuthResponse>()
                                        .await
                                        .map(|auth| auth.token)
                                        .map_err(|e| format!("JSON error: {e}"))
                                } else {
                                    Err(format!("Login failed: {}", resp.status()))
                                }
                            }
                            Err(e) => Err(format!("Request error: {e}")),
                        }
                    },
                    Message::LoginFinished,
                )
            }
            Message::LoginFinished(result) => {
                match result {
                    Ok(token) => {
                        self.config.token = Some(token);
                        save_config(&self.config);
                        self.status_message = "Logged in successfully".to_string();
                        self.identity = "".to_string();
                        self.password = "".to_string();
                    }
                    Err(msg) => {
                        self.status_message = msg;
                    }
                }
                Task::none()
            }
            Message::HotKeyTriggered(id) => {
                let action = if id == self.hotkey_start_id {
                    Some("start")
                } else if id == self.hotkey_stop_id {
                    Some("stop")
                } else {
                    None
                };

                if let Some(action) = action {
                    if let Some(token) = self.config.token.clone() {
                        let domain = self.domain.clone();
                        let action_str = action.to_string();
                        let token_str = token.clone();
                        Task::perform(
                            async move {
                                let url = format!("{}/api/timer/{}", domain, action_str);
                                let client = reqwest::Client::new();
                                match client.post(&url).bearer_auth(token_str).send().await {
                                    Ok(_) => Ok(()),
                                    Err(e) => Err(format!("Timer {} error: {}", action_str, e)),
                                }
                            },
                            move |res| Message::TimerRequestFinished(action.to_string(), res),
                        )
                    } else {
                        self.status_message = "Not logged in".to_string();
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::TimerRequestFinished(action, res) => {
                if let Err(e) = res {
                    self.status_message = e;
                } else {
                    println!("Timer {} successful", action);
                }
                Task::none()
            }
            Message::TrayEvent(event) => {
                if let TrayIconEvent::Click { .. } = event {
                    if let Some(id) = self.window_id {
                        iced::window::set_mode(id, iced::window::Mode::Windowed)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::MenuEvent(event) => {
                if event.id == self.show_item_id {
                    if let Some(id) = self.window_id {
                        iced::window::set_mode(id, iced::window::Mode::Windowed)
                    } else {
                        Task::none()
                    }
                } else if event.id == self.quit_item_id {
                    std::process::exit(0);
                } else {
                    Task::none()
                }
            }
            Message::MinimizeToTray => {
                if let Some(id) = self.window_id {
                    iced::window::set_mode(id, iced::window::Mode::Hidden)
                } else {
                    Task::none()
                }
            }
            Message::WindowOpened(id) => {
                self.window_id = Some(id);
                Task::none()
            }
            Message::StartModifierChanged(modifier) => {
                self.config.start_modifier = modifier;
                self.update_hotkeys();
                Task::none()
            }
            Message::StartKeyChanged(key) => {
                self.config.start_key = key;
                self.update_hotkeys();
                Task::none()
            }
            Message::StopModifierChanged(modifier) => {
                self.config.stop_modifier = modifier;
                self.update_hotkeys();
                Task::none()
            }
            Message::StopKeyChanged(key) => {
                self.config.stop_key = key;
                self.update_hotkeys();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut content = column![
            text("Adventuria Timer").size(30),
            column![
                row![
                    text("Domain:").width(Length::Fixed(80.0)),
                    text_input("Domain", &self.domain).on_input(Message::DomainChanged)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Identity:").width(Length::Fixed(80.0)),
                    text_input("Identity", &self.identity).on_input(Message::IdentityChanged)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Password:").width(Length::Fixed(80.0)),
                    text_input("Password", &self.password)
                        .on_input(Message::PasswordChanged)
                        .secure(true)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(10),
            row![
                button("Login").on_press(Message::LoginPressed),
                button("Minimize to Tray").on_press(Message::MinimizeToTray),
            ]
            .spacing(10),
            text(format!("Status: {}", self.status_message)),
        ]
        .spacing(20)
        .padding(20)
        .max_width(400)
        .align_x(Alignment::Center);

        if self.config.token.is_some() {
            content = content.push(text("Authenticated").color([0.0, 0.5, 0.0]).size(18));
            content = content.push(
                column![
                    text("Hotkeys:").size(18),
                    row![
                        text("Start:").width(Length::Fixed(50.0)),
                        pick_list(
                            &HotkeyModifier::ALL[..],
                            Some(self.config.start_modifier),
                            Message::StartModifierChanged
                        ),
                        text("+"),
                        pick_list(
                            &ALL_KEYS[..],
                            Some(self.config.start_key),
                            Message::StartKeyChanged
                        ),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        text("Stop:").width(Length::Fixed(50.0)),
                        pick_list(
                            &HotkeyModifier::ALL[..],
                            Some(self.config.stop_modifier),
                            Message::StopModifierChanged
                        ),
                        text("+"),
                        pick_list(
                            &ALL_KEYS[..],
                            Some(self.config.stop_key),
                            Message::StopKeyChanged
                        ),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(10),
            );
        }

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
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
                _ => Message::WindowOpened(id),
            }),
        ])
    })
    .run()
}
