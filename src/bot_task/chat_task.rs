use std::time::{Duration, Instant};

use azalea::{Client, Event};

use crate::command_controler::BotTask;

pub struct Chat {
    last_update: Instant,
    messages: Vec<String>,
    index: usize,
}

static DELAY: Duration = Duration::from_secs(1);

impl Chat {
    pub fn init(args: Vec<String>) -> Self {
        Self {
            last_update: Instant::now() - DELAY,
            messages: args,
            index: 0,
        }
    }
}

impl BotTask for Chat {
    fn get_name(&self) -> &str {
        "Chat"
    }
    fn on_event(&mut self, bot: &Client, _event: &Event) {
        if self.last_update.elapsed() >= DELAY {
            bot.chat(self.messages[self.index].as_str());
            self.index += 1;
            self.last_update = Instant::now();
        }
    }

    fn end(&self) -> bool {
        self.index == self.messages.len()
        // Clean up
    }
}
