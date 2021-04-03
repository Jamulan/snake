#[macro_use]
extern crate glium;
extern crate rust_lm;

mod snake;

use rurel::mdp::{Agent, State};
use rurel::strategy::terminate::TerminationStrategy;
use rurel::strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations};
use rurel::AgentTrainer;
use std::hash::{Hash, Hasher};

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum MapState {
    Empty,
    SnakeBody,
    Wall,
}

#[derive(Clone)]
enum Fake {
    Val(f64),
}

impl PartialEq for Fake {
    fn eq(&self, other: &Self) -> bool {
        return true;
    }
}

impl Eq for Fake {}

impl Hash for Fake {
    fn hash<H: Hasher>(&self, state: &mut H) {}
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct MyState {
    map: [MapState; 4],
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
}

impl Agent<MyState> for MyAgent {
    fn current_state(&self) -> &MyState {
        return &self.state;
    }

    fn take_action(&mut self, action: &<MyState as State>::A) {
        self.state.reward = Fake::Val(self.game.tick(*action));
        self.game.render();

        let mut head = (0, 0);
        if let Some(thing) = self.game.snake.get(self.game.snake.len() - 1) {
            head = *thing;
        } else {
            panic!();
        }
        self.state.map = [MapState::Empty; 4];
        for item in self.game.snake.iter() {
            if item.0 - head.0 == 1 && item.1 - head.1 == 0 {
                self.state.map[0] = MapState::SnakeBody;
            }
            if item.0 - head.0 == -1 && item.1 - head.1 == 0 {
                self.state.map[1] = MapState::SnakeBody;
            }
            if item.0 - head.0 == 0 && item.1 - head.1 == 1 {
                self.state.map[2] = MapState::SnakeBody;
            }
            if item.0 - head.0 == 0 && item.1 - head.1 == -1 {
                self.state.map[3] = MapState::SnakeBody;
            }
        }
        if head.0 + 1 > self.game.arena_size.0 {
            self.state.map[0] = MapState::Wall;
        }
        if head.0 - 1 < 0 {
            self.state.map[1] = MapState::Wall;
        }
        if head.1 + 1 > self.game.arena_size.0 {
            self.state.map[2] = MapState::Wall;
        }
        if head.1 - 1 < 0 {
            self.state.map[3] = MapState::Wall;
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

struct NeverStop {}

impl<S: State> TerminationStrategy<S> for NeverStop {
    fn should_stop(&mut self, state: &S) -> bool {
        return false;
    }
}

fn main() {
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let game = snake::Arena::new(&events_loop, (16, 16));

    let mut trainer = AgentTrainer::new();
    let mut agent = MyAgent {
        state: MyState {
            map: [MapState::Empty; 4],
            curr_apple: (0, 0),
            reward: Fake::Val(0.0),
        },
        game: game,
    };
    agent.take_action(&snake::Action::YPos);
    trainer.train(
        &mut agent,
        &QLearning::new(0.2, 0.01, 2.),
        &mut NeverStop {},
        &RandomExploration::new(),
    );
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
