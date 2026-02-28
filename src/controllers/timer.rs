use crate::controllers::Controller;
use crate::{AdventuriaApp, Message};
use iced::Task;
use serde::{Deserialize, Serialize};

pub struct TimerState {
    pub is_active: bool,
}

pub struct TimerController {}

impl Controller for TimerController {
    fn update(state: &mut AdventuriaApp, message: Message) -> Task<Message> {
        match message {
            Message::WindowFocused() => {
                if let (Some(user_id), domain) = (state.config.user_id.clone(), state.domain.clone()) {
                    return Task::perform(
                        async move {
                            Self::get_timer_request(&user_id, &domain)
                                .await
                                .map(|resp| resp.items.first().map(|r| r.is_active).unwrap_or(false))
                                .map_err(|e| e)
                        },
                        Message::TimerStatusFinished,
                    );
                }
            }
            Message::TimerRequestFinished(action, Ok(())) => {
                match action.as_str() {
                    "start" => {
                        state.timer.is_active = true;
                    }
                    "stop" => {
                        state.timer.is_active = false;
                    }
                    _ => {}
                }
            }
            Message::TimerStatusFinished(Ok(is_active)) => {
                state.timer.is_active = is_active;
            }
            Message::TimerStatusFinished(Err(e)) => {
                state.status_message = e;
            }
            _ => {}
        }
        Task::none()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimerResponse {
    pub items: Vec<TimerRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimerRecord {
    pub id: String,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimerRequestQuery {
    pub page: usize,
    pub per_page: usize,
    pub filter: String,
    pub skip_total: usize,
}

impl TimerController {
    async fn get_timer_request(user_id: &str, domain: &str) -> Result<TimerResponse, String> {
        let url = format!("{}/api/collections/timers/records", domain);
        let client = reqwest::Client::new();
        match client
            .get(&url)
            .query(&TimerRequestQuery {
                page: 1,
                per_page: 1,
                filter: format!("user=\"{}\"", user_id),
                skip_total: 1,
            })
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    resp.json::<TimerResponse>()
                        .await
                        .map_err(|e| format!("JSON error: {e}"))
                } else {
                    Err(format!("Timer request failed: {}", resp.status()))
                }
            }
            Err(e) => Err(format!("Request error: {e}")),
        }
    }
}
