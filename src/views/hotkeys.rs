use crate::hotkeys::{HotkeyModifier, ALL_KEYS};
use crate::views::View;
use crate::{AdventuriaApp, HotkeysConfigMessage, Message};
use iced::widget::{column, pick_list, row, text};
use iced::{Alignment, Element, Length};

pub struct HotkeysView {}

impl View for HotkeysView {
    fn view(state: &AdventuriaApp) -> Element<'static, Message> {
        if state.config.token.is_some() {
            column![
                column![
                    text("Hotkeys:").size(18),
                    row![
                        text("Start:").width(Length::Fixed(50.0)),
                        pick_list(
                            &HotkeyModifier::ALL[..],
                            Some(state.config.start_modifier),
                            |m| Message::HotkeysConfig(HotkeysConfigMessage::StartModifierChanged(
                                m
                            ))
                        ),
                        text("+"),
                        pick_list(&ALL_KEYS[..], Some(state.config.start_key), |m| {
                            Message::HotkeysConfig(HotkeysConfigMessage::StartKeyChanged(m))
                        }),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    row![
                        text("Stop:").width(Length::Fixed(50.0)),
                        pick_list(
                            &HotkeyModifier::ALL[..],
                            Some(state.config.stop_modifier),
                            |m| Message::HotkeysConfig(HotkeysConfigMessage::StopModifierChanged(
                                m
                            ))
                        ),
                        text("+"),
                        pick_list(&ALL_KEYS[..], Some(state.config.stop_key), |m| {
                            Message::HotkeysConfig(HotkeysConfigMessage::StopKeyChanged(m))
                        }),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                ]
                .spacing(10)
            ]
            .into()
        } else {
            column![text("Not Authenticated").color([0.8, 0.0, 0.0]).size(18)].into()
        }
    }
}
