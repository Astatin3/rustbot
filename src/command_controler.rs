use azalea::{Client, Event};

use crate::bot_task::*;

pub trait BotTask: Send {
    fn on_event(&mut self, bot: &Client, event: &Event);
    fn end(&self) -> bool;
}

pub fn parse_command(command: &str) -> Option<Box<dyn BotTask>> {
    let args = command
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    match args[0].as_str() {
        "!chat" => Some(Box::new(Chat::init(args[1..].to_vec()))),
        "!goto" => {
            if let Some(task) = GotoBlock::parse(args[1..].to_vec()) {
                Some(Box::new(task))
            } else {
                return None;
            }
        }
        _ => None,
    }
}
