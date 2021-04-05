#[macro_use]
extern crate glium;
extern crate rust_lm;
extern crate rustbreak;

mod snake;

use ahash::AHashMap;
use rurel::mdp::{Agent, State};
use rurel::strategy::terminate::{FixedIterations, TerminationStrategy};
use rurel::strategy::{explore::RandomExploration, learn::QLearning};
use rurel::AgentTrainer;
use rustbreak::backend::PathBackend;
use rustbreak::{deser::Ron, Database, PathDatabase};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::thread::sleep;

pub struct Config {
    pub bound: usize,
    pub arena_size: (i32, i32),
    pub learning: QLearning,
}

pub struct AiComponents {
    config: Config,
    trainer: AgentTrainer<MyState>,
    event_loop: glium::glutin::event_loop::EventLoop<()>,
    agent: snake::Arena,
    db: Database<HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>, PathBackend, Ron>,
}

impl AiComponents {
    pub fn new(config: Config) -> AiComponents {
        if config.bound % 2 == 0 {
            panic!("Config.bound must be odd");
        }
        let db = match PathDatabase::<
            HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>,
            Ron,
        >::load_from_path_or(
            "snake_ai_database.txt".parse().unwrap(), HashMap::default()
        ) {
            Ok(db) => db,
            Err(e) => {
                panic!(e);
            }
        };
        let event_loop = glium::glutin::event_loop::EventLoop::new();
        let agent = snake::Arena::new(config.arena_size, config.bound, &event_loop);

        AiComponents {
            config: config,
            trainer: AgentTrainer::new(),
            event_loop: event_loop,
            agent: agent,
            db: db,
        }
    }

    pub fn train(&mut self, minutes: i32) {
        {
            let bound = self.config.bound;
            let mut vals = HashMap::new();
            self.db.read(|db| {
                if let Some(tmp_vals) = db.get(&bound) {
                    vals = tmp_vals.clone();
                }
            });
            self.trainer.import_state(vals);
        }
        for _ in 0..minutes {
            self.trainer.train(
                &mut self.agent,
                &self.config.learning,
                &mut TimePassed::new(std::time::Duration::from_secs(60)),
                &RandomExploration::new(),
            );
            self.db.write(|db| {
                let exported = self.trainer.export_learned_values();
                db.insert(self.config.bound, exported);
            });
            self.db.save();
        }
    }
}

pub fn test(config: Config) {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut agent = snake::Arena::new(config.arena_size, config.bound, &event_loop);
    let mut curr_action = snake::Action::YPos;

    let db = match PathDatabase::<
        HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>,
        Ron,
    >::load_from_path_or(
        "snake_ai_database.txt".parse().unwrap(), HashMap::default()
    ) {
        Ok(db) => db,
        Err(e) => {
            panic!(e);
        }
    };

    event_loop.run(move |event, thing, control_flow| {
        match event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => {}
            },
            glium::glutin::event::Event::NewEvents(cause) => match cause {
                glium::glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glium::glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        agent.tick(curr_action);

        {
            let state = agent.current_state().clone();
            let mut actions = HashMap::new();
            let mut does_val_exist = false;
            db.read(|db| {
                if let Some(table) = db.get(&config.bound) {
                    if let Some(val) = table.get(&state) {
                        actions = val.clone();
                        does_val_exist = true;
                    }
                }
            });
            if does_val_exist {
                let mut best = (snake::Action::YPos, -10.69);
                for key in actions.keys() {
                    if actions[key] > best.1 {
                        best.0 = *key;
                        best.1 = actions[key];
                    }
                }
                curr_action = best.0;
            } else {
                curr_action = snake::Action::YPos;
                agent.reset();
            }
        }

        agent.render();
    });
}

pub fn play_human(config: Config) {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut game = snake::Arena::new(config.arena_size, config.bound, &event_loop);
    let mut curr_action = snake::Action::YPos;

    event_loop.run(move |event, thing, control_flow| {
        match event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glium::glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.state
                {
                    glium::glutin::event::ElementState::Pressed => match input.scancode {
                        103 => {
                            curr_action = snake::Action::YPos;
                        }
                        108 => {
                            curr_action = snake::Action::YNeg;
                        }
                        105 => {
                            curr_action = snake::Action::XNeg;
                        }
                        106 => {
                            curr_action = snake::Action::XPos;
                        }
                        _ => {
                            println!("{}", input.scancode);
                        }
                    },
                    glium::glutin::event::ElementState::Released => match input.scancode {
                        _ => {}
                    },
                },
                _ => {}
            },
            glium::glutin::event::Event::NewEvents(cause) => match cause {
                glium::glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glium::glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        game.tick(curr_action);

        game.render();
    });
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Fake {
    Val(f64),
}

impl PartialEq for Fake {
    fn eq(&self, _other: &Self) -> bool {
        return true;
    }
}

impl Eq for Fake {}

impl Hash for Fake {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MapState {
    Empty,
    Death,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub struct MyState {
    // is the given tile Death
    pub map: Vec<Vec<MapState>>,
    // indicates the direction towards the apple
    pub curr_apple: (i32, i32),
    pub reward: Fake,
}

impl MyState {
    pub fn new(bound: usize) -> MyState {
        let mut map: Vec<Vec<MapState>> = Vec::with_capacity(bound);
        for i in 0..bound {
            map.push(Vec::with_capacity(bound));
            for j in 0..bound {
                map[i].push(MapState::Empty);
            }
        }
        MyState {
            map: map,
            curr_apple: (0, 0),
            reward: Fake::Val(0.0),
        }
    }
}

impl State for MyState {
    type A = snake::Action;

    fn reward(&self) -> f64 {
        if let Fake::Val(v) = self.reward {
            return v;
        } else {
            panic!();
        }
    }

    fn actions(&self) -> Vec<Self::A> {
        vec![
            snake::Action::XPos,
            snake::Action::YPos,
            snake::Action::XNeg,
            snake::Action::YNeg,
        ]
    }
}

impl Agent<MyState> for snake::Arena {
    fn current_state(&self) -> &MyState {
        return &self.state;
    }

    fn take_action(&mut self, action: &<MyState as State>::A) {
        self.tick(*action);
    }
}

struct TimePassed {
    termination_time: std::time::Instant,
}

impl TimePassed {
    pub fn new(time_to_train: std::time::Duration) -> TimePassed {
        TimePassed {
            termination_time: std::time::Instant::now() + time_to_train,
        }
    }
}

impl<S: State> TerminationStrategy<S> for TimePassed {
    fn should_stop(&mut self, _state: &S) -> bool {
        if let Option::Some(_) =
            std::time::Instant::now().checked_duration_since(self.termination_time)
        {
            true
        } else {
            false
        }
    }
}

struct NumGames {
    curr_game: i32,
    target_games: i32,
}

impl NumGames {
    pub fn new(target_games: i32) -> NumGames {
        return NumGames {
            curr_game: 0,
            target_games: target_games,
        };
    }
}

impl<S: State> TerminationStrategy<S> for NumGames {
    fn should_stop(&mut self, state: &S) -> bool {
        if state.reward() < -0.5 {
            self.curr_game += 1;
        }
        return self.curr_game == self.target_games;
    }
}
