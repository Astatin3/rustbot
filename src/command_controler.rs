use azalea::{Client, Event};
use std::fmt::Debug;

pub trait BotTask: Send {
    fn get_name(&self) -> &str;
    fn on_event(&mut self, bot: &Client, event: &Event);
    fn end(&self) -> bool;
}

impl Debug for dyn BotTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BotTask({})", self.get_name())
    }
}
