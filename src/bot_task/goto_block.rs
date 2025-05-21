use azalea::{
    BlockPos, Client, Event,
    pathfinder::{Pathfinder, goals},
    prelude::PathfinderClientExt,
};

use crate::command_controler::BotTask;

pub struct GotoBlock {
    x: i32,
    y: Option<i32>,
    z: i32,
    started: bool,
    finished: bool,
}

impl GotoBlock {
    pub fn parse(args: Vec<String>) -> Option<Self> {
        if args.len() == 3 {
            let x = args[0].parse().ok()?;
            let y = args[1].parse().ok()?;
            let z = args[2].parse().ok()?;
            Some(Self {
                x,
                y: Some(y),
                z,
                started: false,
                finished: false,
            })
        } else if args.len() == 2 {
            let x = args[0].parse().ok()?;
            let z = args[1].parse().ok()?;
            Some(Self {
                x,
                y: None,
                z,
                started: false,
                finished: false,
            })
        } else {
            return None;
        }
    }
}

impl BotTask for GotoBlock {
    fn on_event(&mut self, bot: &Client, event: &Event) {
        if !self.started {
            self.started = true;
            bot.chat(
                format!("Going to ({}, {}, {})", self.x, self.y.unwrap_or(0), self.z).as_str(),
            );
            if let Some(y) = self.y {
                bot.start_goto(goals::BlockPosGoal(BlockPos {
                    x: self.x,
                    y,
                    z: self.z,
                }));
            } else {
                bot.start_goto(goals::XZGoal {
                    x: self.x,
                    z: self.z,
                });
            }
        } else {
            match event {
                Event::Tick => {
                    self.finished = {
                        let pos = bot.position().to_block_pos_floor();

                        pos.x == self.x
                            && pos.z == self.z
                            && (self.y.is_none() || pos.y == self.y.unwrap())
                    }
                }
                _ => {}
            }
        }
    }

    fn end(&self) -> bool {
        // bot.chat("Arrived!")
        self.finished
    }
}
