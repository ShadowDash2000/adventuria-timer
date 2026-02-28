use crate::views::hotkeys::HotkeysView;
use crate::views::timer::TimerView;
use crate::views::View;
use crate::{AdventuriaApp, Message};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

pub struct MainView {}

impl View for MainView {
    fn view(state: &AdventuriaApp) -> Element<'static, Message> {
        let mut content = column![
            text("Adventuria Timer").size(30),
            column![
                row![
                    text("Domain:").width(Length::Fixed(80.0)),
                    text_input("Domain", &state.domain).on_input(Message::DomainChanged)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Identity:").width(Length::Fixed(80.0)),
                    text_input("Identity", &state.identity).on_input(Message::IdentityChanged)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Password:").width(Length::Fixed(80.0)),
                    text_input("Password", &state.password)
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
            text(format!("Status: {}", state.status_message)),
        ]
        .spacing(20)
        .padding(20)
        .max_width(400)
        .align_x(Alignment::Center);

        content = content.push(TimerView::view(state));
        content = content.push(HotkeysView::view(state));

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}
