use crate::{AdventuriaApp, Message};
use iced::Task;

pub trait Controller {
    fn update(state: &mut AdventuriaApp, message: Message) -> Task<Message>;
}
