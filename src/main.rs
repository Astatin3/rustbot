use std::{alloc::System, sync::Arc};

pub mod bot_task;
pub mod command_controler;

use azalea::{
    prelude::*,
    swarm::{Swarm, SwarmBuilder, SwarmEvent},
};
use command_controler::BotTask;
use parking_lot::Mutex;

use crate::bot_task::*;

pub static BOT_COUNT: usize = 3;
pub static BOT_PREFIX: &'static str = "bot";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SwarmBuilder::new()
        .add_accounts(
            (0..BOT_COUNT)
                .map(|i| Account::offline(format!("{}{}", BOT_PREFIX, i).as_str()))
                .collect(),
        )
        .set_handler(handle)
        .set_swarm_handler(handle_swarm)
        .start("localhost")
        .await?

    // ClientBuilder::new()
    //     .set_handler(handle)
    //     .set_state(BotState {
    //         commands: Arc::new(Mutex::new(Vec::new())),
    //     })
    //     .start(Account::offline("bot"), "localhost")
    //     .await?;
}

async fn handle(bot: Client, event: Event, state: BotState) -> anyhow::Result<()> {
    if state.task.lock().is_some() {
        // Process commands
        if state.task.lock().as_ref().unwrap().end() {
            bot.chat(
                format!(
                    "Finished {}",
                    print_type_of(&state.task.lock().as_ref().unwrap())
                )
                .as_str(),
            );
            *state.task.lock() = None;
        } else {
            state.task.lock().as_mut().unwrap().on_event(&bot, &event);
        }
    } else {
        let swarm_state = bot.resource::<SwarmState>();
        if !swarm_state.tasks.lock().is_empty() {
            *state.task.lock() = Some(swarm_state.tasks.lock().remove(0));
            bot.chat(
                format!(
                    "Starting {}",
                    print_type_of(&state.task.lock().as_ref().unwrap())
                )
                .as_str(),
            );
        }
    }

    Ok(())
}

#[derive(Clone, Component)]
pub struct BotState {
    pub task: Arc<Mutex<Option<Box<dyn BotTask>>>>,
    // pub messages_received: Arc<Mutex<usize>>,
}

impl Default for BotState {
    fn default() -> Self {
        Self {
            task: Arc::new(Mutex::new(None)),
        }
    }
}

impl BotState {
    fn get_task(&self) -> String {
        match self.task.lock().as_ref() {
            Some(task) => task.get_name().to_string(),
            None => "No task".to_string(),
        }
    }
}

#[derive(Resource, Default, Clone)]
struct SwarmState {
    pub tasks: Arc<Mutex<Vec<Box<dyn BotTask>>>>,
}

async fn handle_swarm(swarm: Swarm, event: SwarmEvent, state: SwarmState) -> anyhow::Result<()> {
    match &event {
        SwarmEvent::Init => {
            println!("Swarm initialized");
        }
        SwarmEvent::Login => {
            println!("All bots have logged in");
        }
        SwarmEvent::Disconnect(account, join_opts) => {
            println!("Bot {} disconnected", account.username);
            swarm
                .add_with_opts(account, BotState::default(), join_opts)
                .await?;
        }
        SwarmEvent::Chat(msg) => {
            let command = msg.content();
            println!("Chat message: {}", command);

            println!("{}", command);
            if let Some(command) = {
                let args = command
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                match args[0].as_str() {
                    "!chat" => Some(Box::new(Chat::init(args[1..].to_vec())) as Box<dyn BotTask>),
                    "!goto" => {
                        if let Some(task) = GotoBlock::parse(args[1..].to_vec()) {
                            Some(Box::new(task) as Box<dyn BotTask>)
                        } else {
                            None
                        }
                    }
                    "!status" => {
                        for bot in swarm {
                            let botstate = &bot.get_component::<BotState>().unwrap();
                            println!("{} - {}", bot.username(), botstate.get_task());
                        }
                        println!("Unstarted: {:?}", state.tasks.lock());

                        None
                    }
                    _ => None,
                }
            } {
                state.tasks.lock().push(command);
            }

            // match state.commands.execute(
            //     command,
            //     Mutex::new(CommandSource {
            //         bot: bot.clone(),
            //         chat: chat.clone(),
            //         state: state.clone(),
            //     }),
            // ) {
            //     Ok(_) => {}
            //     Err(err) => {
            //         eprintln!("{err:?}");
            //         let command_source = CommandSource {
            //             bot,
            //             chat: chat.clone(),
            //             state: state.clone(),
            //         };
            //         command_source.reply(&format!("{err:?}"));
            //     }
            // }
        }
        _ => {}
    }
    Ok(())
}

fn print_type_of<T>(_: &T) -> String {
    std::any::type_name::<T>().to_string()
}
