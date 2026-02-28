use crate::views::View;
use crate::{AdventuriaApp, Message};
use iced::widget::{column, row, text};
use iced::Element;

pub struct TimerView {}

impl View for TimerView {
    fn view(state: &AdventuriaApp) -> Element<'static, Message> {
        if state.config.token.is_some() {
            let (status_text, status_color) = if state.timer.is_active {
                ("Active", iced::Color::from_rgb(0.0, 0.8, 0.0))
            } else {
                ("Paused", iced::Color::from_rgb(0.8, 0.0, 0.0))
            };

            column![row![
                text("Timer: "),
                text(status_text).color(status_color)
            ]]
            .into()
        } else {
            column!().into()
        }
    }
}
