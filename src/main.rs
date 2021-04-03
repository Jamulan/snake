#[macro_use]
extern crate glium;
extern crate rust_lm;

mod snake;

use rurel::mdp::{Agent, State};
use rurel::strategy::terminate::TerminationStrategy;
use rurel::strategy::{explore::RandomExploration, learn::QLearning, terminate::FixedIterations};
use rurel::AgentTrainer;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum MapState {
    Empty,
    SnakeBody,
    SnakeHead,
    Apple,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct MyState {
    map: [[MapState; 64]; 64],
}

impl State for MyState {
    type A = snake::Action;

    fn reward(&self) -> f64 {
        let mut acc = 0.0f64;
        for thing in self.map.iter() {
            for item in thing.iter() {
                if let MapState::SnakeBody = *item {
                    acc += 1.0f64;
                }
            }
        }
        return acc - 2.0f64;
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
        self.game.tick(*action);
        self.game.render();

        let mut last = (0, 0);
        for item in self.game.snake.iter() {
            last = (item.0 as usize, item.1 as usize);
            self.state.map[last.0][last.1] = MapState::SnakeBody;
        }
        self.state.map[last.0][last.1] = MapState::SnakeHead;
        self.state.map[self.game.apple_pos.0 as usize][self.game.apple_pos.1 as usize] =
            MapState::Apple;
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
    let game = snake::Arena::new(&events_loop, (64, 64));

    let mut trainer = AgentTrainer::new();
    let mut agent = MyAgent {
        state: MyState {
            map: [[MapState::Empty; 64]; 64],
        },
        game: game,
    };
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
