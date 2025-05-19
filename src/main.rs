use std::{collections::HashSet, sync::Arc, thread::sleep, time::Duration};

use azalea::{
    BlockPos, Bot, GameProfileComponent,
    blocks::{BlockState, properties::Type},
    ecs::{entity::Entity, query::With},
    entity::{Position, metadata::Player},
    pathfinder::{GotoEvent, Pathfinder, astar::PathfinderTimeout, goals, moves::default_move},
    prelude::*,
    registry::{Block, Item},
    world::find_blocks::FindBlocks,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
// use parking_lot::Mutex;

#[tokio::main]
async fn main() {
    let account = Account::offline("bot");
    // or Account::microsoft("example@example.com").await.unwrap();

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, "localhost")
        .await
        .unwrap();
}

#[derive(Clone)]
pub enum BotAction {
    Nothing,
    Goto(Position),
    PathfindMine(PathfindMineAction),
}

#[derive(Clone)]
pub enum PathfindMineAction {
    Start,
    Goto(HashSet<BlockState>, BlockPos),
    Mine(HashSet<BlockState>, BlockPos),
}

impl Default for BotAction {
    fn default() -> Self {
        Self::Nothing
    }
}

pub static MINE_DISTANCE: i32 = 7;

lazy_static! {
    pub static ref MINE_BLOCKS: HashSet<BlockState> = {
        let mut b = HashSet::new();

        b.insert(Block::OakLog.into());

        b
    };
}

#[derive(Default, Clone, Component)]
pub struct State {
    pub action: Arc<Mutex<BotAction>>,
    // pub messages_received: Arc<Mutex<usize>>,
}

async fn handle(bot: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::Chat(m) => {
            let (sender, message) = m.split_sender_and_content();
            println!("<{:?}> {}", sender, message);

            if sender.is_some() && message.starts_with("!") {
                match message.as_str() {
                    "!follow" => {
                        let uuid = m.sender_uuid().unwrap();
                        let entity = bot.entity_by_uuid(uuid).unwrap();
                        let position = bot.get_entity_component::<Position>(entity).unwrap();

                        bot.chat(format!("Following player {:?}", position).as_str());

                        bot.ecs.lock().send_event(GotoEvent {
                            entity: bot.entity,
                            goal: Arc::new(goals::BlockPosGoal(position.to_block_pos_ceil())),
                            successors_fn: default_move,
                            allow_mining: true,
                            min_timeout: PathfinderTimeout::Time(Duration::from_secs(2)),
                            max_timeout: PathfinderTimeout::Time(Duration::from_secs(10)),
                        });
                    }
                    "!mine" => {
                        *state.action.lock() = BotAction::PathfindMine(PathfindMineAction::Start);
                        std::mem::drop(state.action);

                        // blocks.
                        //
                    }
                    "!stop" => {
                        bot.stop_pathfinding();
                        *state.action.lock() = BotAction::Nothing;
                        std::mem::drop(state.action);
                    }
                    _ => {
                        bot.chat(format!("Invalid command: {}", message).as_str());
                    }
                }
            }

            // *state.messages_received.lock() += 1;
        }
        Event::Tick => {
            let action = state.action.lock().clone();
            match action {
                BotAction::PathfindMine(action) => match action {
                    PathfindMineAction::Start => {
                        let block = bot
                            .world()
                            .read()
                            .find_blocks(
                                bot.position(),
                                &azalea::blocks::BlockStates {
                                    set: MINE_BLOCKS.clone(),
                                },
                            )
                            .next()
                            .unwrap();

                        bot.ecs.lock().send_event(GotoEvent {
                            entity: bot.entity,
                            goal: Arc::new(goals::RadiusGoal {
                                pos: block.center(),
                                radius: MINE_DISTANCE as f32,
                            }),
                            successors_fn: default_move,
                            allow_mining: true,
                            min_timeout: PathfinderTimeout::Time(Duration::from_secs(2)),
                            max_timeout: PathfinderTimeout::Time(Duration::from_secs(10)),
                        });
                        std::mem::drop(bot.ecs);

                        // println!("Starting goto start...");

                        *state.action.lock() = BotAction::PathfindMine(PathfindMineAction::Goto(
                            MINE_BLOCKS.clone(),
                            block,
                        ));
                        std::mem::drop(state.action);
                        // println!("Finished lock");
                    }
                    PathfindMineAction::Goto(blocks, block) => {
                        if is_goto_target_reached(&bot) {
                            bot.start_mining(block);

                            // println!("Starting mine...");

                            *state.action.lock() = BotAction::PathfindMine(
                                PathfindMineAction::Mine(MINE_BLOCKS.clone(), block.clone()),
                            );
                            std::mem::drop(state.action);
                            // println!("Droped mine...");
                        }
                    }
                    PathfindMineAction::Mine(blocks, block) => {
                        println!("Stall 1");
                        if let Some(blockstate) = bot.world().read().get_block_state(&block) {
                            println!("Stall 2");
                            if blockstate.is_air() {
                                println!("Stall 3");
                                if let Some(block) = bot
                                    .world()
                                    .read()
                                    .find_blocks(
                                        bot.position(),
                                        &azalea::blocks::BlockStates {
                                            set: MINE_BLOCKS.clone(),
                                        },
                                    )
                                    .next()
                                {
                                    println!("Stall 4");
                                    // {
                                    //     println!("Stall 5");
                                    //     bot.ecs.lock()
                                    // }
                                    // .send_event(GotoEvent {
                                    //     entity: {
                                    //         println!("Stall 6");
                                    //         bot.entity
                                    //     },
                                    //     goal: Arc::new(goals::RadiusGoal {
                                    //         pos: block.center(),
                                    //         radius: MINE_DISTANCE as f32,
                                    //     }),
                                    //     successors_fn: default_move,
                                    //     allow_mining: true,
                                    //     min_timeout: PathfinderTimeout::Time(Duration::from_secs(
                                    //         2,
                                    //     )),
                                    //     max_timeout: PathfinderTimeout::Time(Duration::from_secs(
                                    //         10,
                                    //     )),
                                    // });
                                    // println!("Stall 7");
                                    //
                                    // std::mem::drop(bot.ecs);

                                    sleep(Duration::from_millis(100));

                                    bot.goto(goals::RadiusGoal {
                                        pos: block.center(),
                                        radius: MINE_DISTANCE as f32,
                                    });

                                    // bot.is_go().await;

                                    println!("Stall 82");

                                    println!("Starting Goto...");
                                    *state.action.lock() = BotAction::PathfindMine(
                                        PathfindMineAction::Goto(MINE_BLOCKS.clone(), block),
                                    );
                                    std::mem::drop(state.action);
                                    println!("Dropped Goto...");
                                } else {
                                    *state.action.lock() = BotAction::Nothing;
                                    std::mem::drop(state.action);
                                }
                            }
                        }
                    }
                },

                _ => {}
            }
        }
        _ => {}
    }

    Ok(())
}

fn is_goto_target_reached(bot: &Client) -> bool {
    bot.map_get_component::<Pathfinder, _>(|p| {
        p.map(|p| p.goal.is_none() && !p.is_calculating)
            .unwrap_or(true)
    })
}

fn player_by_name(bot: &Client, name: String) -> Option<Entity> {
    bot.entity_by::<With<Player>, (&GameProfileComponent,)>(
        |(profile,): &(&GameProfileComponent,)| {
            // return sender.unwrap() == profile.name;
            profile.name == name
        },
    )
}
