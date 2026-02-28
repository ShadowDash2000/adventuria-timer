use iced::Element;
use crate::{AdventuriaApp, Message};

pub trait View {
    fn view(state: &AdventuriaApp) -> Element<'static, Message>;
}