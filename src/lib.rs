#[macro_use]
extern crate glium;
extern crate rust_lm;
extern crate rustbreak;

mod snake;

use rurel::mdp::{Agent, State};
use rurel::strategy::terminate::TerminationStrategy;
use rurel::strategy::{explore::RandomExploration, learn::QLearning};
use rurel::AgentTrainer;
use rustbreak::backend::PathBackend;
use rustbreak::{deser::Ron, Database, PathDatabase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub struct Config {
    pub bound: usize,
    pub arena_size: (i32, i32),
    pub learning: QLearning,
    pub render: bool,
}

pub struct AiComponents {
    config: Config,
    trainer: AgentTrainer<MyState>,
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
            "snake_ai_database.ron".parse().unwrap(), HashMap::default()
        ) {
            Ok(db) => db,
            Err(e) => {
                println!("{:?}", e);
                panic!();
            }
        };
        let event_loop = glium::glutin::event_loop::EventLoop::new();
        let agent = snake::Arena::new(config.arena_size, config.bound, &event_loop, config.render);

        AiComponents {
            config: config,
            trainer: AgentTrainer::new(),
            agent: agent,
            db: db,
        }
    }

    pub fn train(&mut self) {
        self.train_for_time(0);
    }

    pub fn train_for_time(&mut self, minutes: u32) {
        let rand_explore = &RandomExploration::new();
        let mut i = 0;
        loop {
            self.load();
            self.trainer.train(
                &mut self.agent,
                &self.config.learning,
                &mut TimePassed::new(std::time::Duration::from_secs(60)),
                rand_explore,
            );
            self.save();

            i += 1;
            if i == minutes {
                return;
            }
        }
    }

    pub fn train_for_games(&mut self, games: u32) {
        self.load();
        self.trainer.train(
            &mut self.agent,
            &self.config.learning,
            &mut NumGames::new(games),
            &RandomExploration::new(),
        );
        self.save();
    }

    fn load(&mut self) {
        load_db(&self.db, &mut self.trainer, self.config.bound);
    }
    fn save(&mut self) {
        save_db(&self.db, &mut self.trainer, self.config.bound);
    }
}

pub fn test(config: Config) {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut agent = snake::Arena::new(config.arena_size, config.bound, &event_loop, config.render);
    let mut curr_action = snake::Action::YPos;

    let db = match PathDatabase::<
        HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>,
        Ron,
    >::load_from_path_or(
        "snake_ai_database.ron".parse().unwrap(), HashMap::default(),
    ) {
        Ok(db) => db,
        Err(e) => {
            println!("{:?}", e);
            panic!();
        }
    };
    let mut trainer = AgentTrainer::new();
    load_db(&db, &mut trainer, config.bound);
    let mut time_start = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glium::glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.state
                {
                    glium::glutin::event::ElementState::Pressed => match input.scancode {
                        28 => {
                            load_db(&db, &mut trainer, config.bound);
                            agent.reset();
                            return;
                        }
                        _ => {
                            println!("{}", input.scancode);
                        }
                    },
                    glium::glutin::event::ElementState::Released => match input.scancode {
                        _ => {}
                    },
                },
                _ => return,
            },
            glium::glutin::event::Event::NewEvents(cause) => match cause {
                glium::glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glium::glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667 * 2);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        if let Some(action) = trainer.best_action(&agent.state) {
            curr_action = action;
        }

        if agent.tick(curr_action) || time_start.elapsed().as_secs() > 60 {
            load_db(&db, &mut trainer, config.bound);
            time_start = std::time::Instant::now();
        }
    });
}

fn save_db(
    db: &Database<HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>, PathBackend, Ron>,
    trainer: &mut AgentTrainer<MyState>,
    bound: usize,
) {
    if let Err(e) = db.write(|db| {
        let exported = trainer.export_learned_values();
        db.insert(bound, exported);
    }) {
        println!("{:?}", e);
        panic!();
    }
    if let Err(e) = db.save() {
        println!("{:?}", e);
        panic!();
    }
}

fn load_db(
    db: &Database<HashMap<usize, HashMap<MyState, HashMap<snake::Action, f64>>>, PathBackend, Ron>,
    trainer: &mut AgentTrainer<MyState>,
    bound: usize,
) {
    if let Err(e) = db.load() {
        println!("{:?}", e);
        panic!();
    }
    let mut vals = HashMap::new();
    if let Err(e) = db.read(|db| {
        if let Some(tmp_vals) = db.get(&bound) {
            vals = tmp_vals.clone();
        }
    }) {
        println!("{:?}", e);
        panic!();
    }
    trainer.import_state(vals);
}

pub fn play_human(config: Config) {
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let mut game = snake::Arena::new(config.arena_size, config.bound, &event_loop, config.render);
    let mut curr_action = snake::Action::YPos;

    event_loop.run(move |event, _, control_flow| {
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
                _ => return,
            },
            glium::glutin::event::Event::NewEvents(cause) => match cause {
                glium::glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glium::glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667 * 4);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        game.tick(curr_action);
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
            for _ in 0..bound {
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
        let Fake::Val(v) = self.reward;
        return v;
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
    curr_game: u32,
    target_games: u32,
}

impl NumGames {
    pub fn new(target_games: u32) -> NumGames {
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
