#[macro_use]
extern crate glium;
extern crate rust_lm;
extern crate rustbreak;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::thread::sleep;

use rurel::mdp::{Agent, State};
use rurel::strategy::terminate::{FixedIterations, TerminationStrategy};
use rurel::strategy::{explore::RandomExploration, learn::QLearning};
use rurel::AgentTrainer;
use rustbreak::backend::PathBackend;
use rustbreak::{deser::Ron, Database, PathDatabase};
use serde::{Deserialize, Serialize};

use crate::snake::Action;

mod snake;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
enum Fake {
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
enum MapState {
    Empty,
    Death,
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
struct MyState {
    // is the given tile Death
    map: [[MapState; 3]; 3],
    // indicates the direction towards the apple
    curr_apple: (i32, i32),
    reward: Fake,
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

struct MyAgent {
    state: MyState,
    game: snake::Arena,
    render: bool,
}

impl Agent<MyState> for MyAgent {
    fn current_state(&self) -> &MyState {
        return &self.state;
    }

    fn take_action(&mut self, action: &<MyState as State>::A) {
        self.state.reward = Fake::Val(self.game.tick(*action));
        if self.render {
            self.game.render();
            sleep(std::time::Duration::from_secs_f64(0.04));
        }

        let mut head = (0, 0);
        if let Some(thing) = self.game.snake.get(self.game.snake.len() - 1) {
            head = *thing;
        } else {
            panic!();
        }
        // populate self.state.map
        {
            self.state.map = [[MapState::Empty; 3]; 3];
            let bounds = (self.state.map.len() as i32, self.state.map[0].len() as i32);
            let local_head = (bounds.0 / 2, bounds.1 / 2);
            for i in 0..bounds.0 {
                for j in 0..bounds.1 {
                    if i != local_head.0 && j != local_head.1 {
                        self.state.map[i as usize][j as usize] = MapState::Empty;
                        continue;
                    }
                    let test = (i - local_head.0 + head.0, j - local_head.1 + head.1);
                    for item in self.game.snake.iter() {
                        if item.0 == test.0 && item.1 == test.1 {
                            self.state.map[i as usize][j as usize] = MapState::Death;
                            break;
                        }
                    }
                    if test.0 < 0
                        || test.1 < 0
                        || test.0 >= self.game.arena_size.0
                        || test.1 >= self.game.arena_size.1
                    {
                        self.state.map[i as usize][j as usize] = MapState::Death;
                    }
                }
            }
        }

        self.state.curr_apple = (
            head.0 - self.game.apple_pos.0,
            head.1 - self.game.apple_pos.1,
        );
        if self.state.curr_apple.0 > 0 {
            self.state.curr_apple.0 = 1;
        } else if self.state.curr_apple.0 < 0 {
            self.state.curr_apple.0 = -1;
        }
        if self.state.curr_apple.1 > 0 {
            self.state.curr_apple.1 = 1;
        } else if self.state.curr_apple.1 < 0 {
            self.state.curr_apple.1 = -1;
        }
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

fn main() {
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let game = snake::Arena::new(&events_loop, (16, 16));

    let mut trainer = AgentTrainer::new();
    let mut agent = MyAgent {
        state: MyState {
            map: [[MapState::Empty; 3]; 3],
            curr_apple: (0, 0),
            reward: Fake::Val(0.0),
        },
        game: game,
        render: false,
    };
    let db =
        match PathDatabase::<HashMap<MyState, HashMap<snake::Action, f64>>, Ron>::load_from_path_or(
            format!("trained_hash_table_{}.txt", agent.state.map.len())
                .parse()
                .unwrap(),
            HashMap::new(),
        ) {
            Ok(db) => db,
            Err(e) => {
                panic!(e);
            }
        };
    db.read(|db| {
        trainer.import_state(db.clone());
    });
    agent.take_action(&snake::Action::YPos);
    for _ in 0..1 {
        trainer.train(
            &mut agent,
            &QLearning::new(0.2, 0.01, 2.),
            &mut TimePassed::new(std::time::Duration::from_secs(60 * 1)),
            &RandomExploration::new(),
        );
        save_db(&db, &trainer);
    }

    println!("TRAINING FINISHED -----");

    agent.render = true;

    events_loop.run(move |event, _, control_flow| {
        match event {
            glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                glium::glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glium::glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glium::glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.state
                {
                    glium::glutin::event::ElementState::Pressed => match input.scancode {
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

        if let Option::Some(action) = trainer.best_action(agent.current_state()) {
            agent.take_action(&action);
        } else {
            trainer.train(
                &mut agent,
                &QLearning::new(0.1, 0.1, 2.),
                &mut NumGames::new(1),
                &RandomExploration::new(),
            );
            save_db(&db, &trainer);
            // println!("MARK ----- ----- ----- -----");
        }
    });
}

fn save_db(
    db: &Database<HashMap<MyState, HashMap<snake::Action, f64>>, PathBackend, Ron>,
    trainer: &AgentTrainer<MyState>,
) {
    let exported = trainer.export_learned_values();
    db.write(|db| {
        for key in exported.keys() {
            db.insert(*key, exported[key].clone());
        }
    });
    if let Err(e) = db.save() {
        panic!(e);
    }
}

fn run_human_playable() {
    let mut curr_action: snake::Action = snake::Action::XNeg;
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let mut game = snake::Arena::new(&events_loop, (64, 64));

    events_loop.run(move |event, _, control_flow| {
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
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glium::glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        game.tick(curr_action);
        game.render();
    });
}
