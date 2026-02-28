use crate::config::save_config;
use crate::controllers::Controller;
use crate::{AdventuriaApp, Message};
use iced::Task;
use serde::{Deserialize, Serialize};
use tray_icon::TrayIconEvent;

pub struct MainController {}

impl Controller for MainController {
    fn update(state: &mut AdventuriaApp, message: Message) -> Task<Message> {
        match message {
            Message::DomainChanged(domain) => {
                state.domain = domain;
                Task::none()
            }
            Message::IdentityChanged(identity) => {
                state.identity = identity;
                Task::none()
            }
            Message::PasswordChanged(password) => {
                state.password = password;
                Task::none()
            }
            Message::LoginPressed => {
                state.status_message = "Logging in...".to_string();
                let domain = state.domain.clone();
                let identity = state.identity.clone();
                let password = state.password.clone();

                Task::perform(
                    async move {
                        MainController::login_request(
                            domain.as_str(),
                            identity.as_str(),
                            password.as_str(),
                        )
                        .await
                    },
                    Message::LoginFinished,
                )
            }
            Message::LoginFinished(result) => {
                match result {
                    Ok(res) => {
                        state.config.token = Some(res.token);
                        state.config.user_id = Some(res.record.id);
                        save_config(&state.config);
                        state.status_message = "Logged in successfully".to_string();
                        state.identity = "".to_string();
                        state.password = "".to_string();
                    }
                    Err(msg) => {
                        state.status_message = msg;
                    }
                }
                Task::none()
            }
            Message::HotKeyTriggered(id) => {
                let action = if id == state.hotkey_start_id {
                    Some("start")
                } else if id == state.hotkey_stop_id {
                    Some("stop")
                } else {
                    None
                };

                if let Some(action) = action {
                    if let Some(token) = state.config.token.clone() {
                        let domain = state.domain.clone();
                        let action_str = action.to_string();
                        let token_str = token.clone();
                        Task::perform(
                            async move {
                                MainController::timer_action_request(
                                    action_str.as_str(),
                                    domain.as_str(),
                                    token_str.as_str(),
                                )
                                .await
                            },
                            move |res| Message::TimerRequestFinished(action.to_string(), res),
                        )
                    } else {
                        state.status_message = "Not logged in".to_string();
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::TimerRequestFinished(action, res) => {
                if let Err(e) = res {
                    state.status_message = e;
                } else {
                    println!("Timer {} successful", action);
                }
                Task::none()
            }
            Message::TrayEvent(event) => {
                if let TrayIconEvent::Click { .. } = event {
                    if let Some(id) = state.window_id {
                        iced::window::set_mode(id, iced::window::Mode::Windowed)
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::MenuEvent(event) => {
                if event.id == state.show_item_id {
                    if let Some(id) = state.window_id {
                        iced::window::set_mode(id, iced::window::Mode::Windowed)
                    } else {
                        Task::none()
                    }
                } else if event.id == state.quit_item_id {
                    std::process::exit(0);
                } else {
                    Task::none()
                }
            }
            Message::MinimizeToTray => {
                if let Some(id) = state.window_id {
                    iced::window::set_mode(id, iced::window::Mode::Hidden)
                } else {
                    Task::none()
                }
            }
            Message::WindowOpened(id) => {
                state.window_id = Some(id);
                Task::none()
            }
            _ => Task::none(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthResponse {
    pub token: String,
    pub record: UserRecord,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserRecord {
    pub id: String,
}

impl MainController {
    async fn login_request(domain: &str, identity: &str, password: &str) -> Result<AuthResponse, String> {
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
                        .map_err(|e| format!("JSON error: {e}"))
                } else {
                    Err(format!("Login failed: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Request error: {e}")),
        }
    }

    async fn timer_action_request(action: &str, domain: &str, token: &str) -> Result<(), String> {
        let url = format!("{}/api/timer/{}", domain, action);
        let client = reqwest::Client::new();
        match client.post(&url).bearer_auth(token).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Timer {} error: {}", action, e)),
        }
    }
}
