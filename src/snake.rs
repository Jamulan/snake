use crate::{Fake, MapState, MyState};
use glium::{glutin, Surface};
use rand::Rng;
use rust_lm::Mat4;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    XPos,
    XNeg,
    YPos,
    YNeg,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    vec_color: (f32, f32, f32),
}

pub struct Arena {
    // (x, y, distance_from_head)
    pub snake: Vec<(i32, i32)>,
    // (x, y, is_spawned)
    pub apple_pos: (i32, i32, bool),
    pub arena_size: (i32, i32),
    pub reward_for_last_action: f64,
    pub state: MyState,
    bound: usize,
    display: Option<glium::Display>,
    program: Option<glium::Program>,
    transform_matrix: Mat4,
    render: bool,
}

impl Arena {
    pub fn new(arena_size: (i32, i32), bound: usize) -> Arena {
        let transform_matrix = Mat4::identity()
            .scale_by(
                2.0 / (arena_size.0 as f32),
                2.0 / (arena_size.1 as f32),
                1.0,
            )
            .translate_by(-1.0, -1.0, 0.0);

        let mut out = Arena {
            snake: Vec::new(),
            apple_pos: (0, 0, false),
            arena_size: arena_size,
            reward_for_last_action: 0.0,
            state: MyState::new(bound),
            bound: bound,
            display: None,
            program: None,
            transform_matrix: transform_matrix,
            render: false,
        };
        out.reset();
        return out;
    }

    pub fn new_render(
        arena_size: (i32, i32),
        bound: usize,
        events_loop: &glutin::event_loop::EventLoop<()>,
    ) -> Arena {
        let wb = glium::glutin::window::WindowBuilder::new()
            .with_inner_size(glium::glutin::dpi::LogicalSize::new(640.0, 640.0))
            .with_title("snake");
        let cb = glium::glutin::ContextBuilder::new().with_vsync(true);
        let display = glium::Display::new(wb, cb, events_loop).unwrap();

        implement_vertex!(Vertex, position, vec_color);

        let vertex_shader_src = r#"
        #version 140
        in vec2 position;
        in vec3 vec_color;
        out vec3 my_color;
        uniform mat4 matrix;
        void main() {
            my_color = vec_color;
            gl_Position = matrix * vec4(position, 0.0, 1.0);
        }
    "#;

        let fragment_shader_src = r#"
        #version 140
        in vec3 my_color;
        out vec4 color;
        void main() {
            color = vec4(my_color, 1.0);
        }
    "#;

        let program =
            glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        let mut out = Self::new(arena_size, bound);
        out.display = Some(display);
        out.program = Some(program);
        out.render = true;
        return out;
    }

    fn new_snake(&mut self) {
        let mut new_snake = Vec::new();
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 2));
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 1));
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 0));
        self.snake = new_snake;
    }

    pub fn reset(&mut self) {
        // println!("length at death: {}", self.snake.len());
        self.new_snake();
        self.gen_apple();
    }

    fn gen_apple(&mut self) {
        self.apple_pos.2 = false;
        loop {
            let test: (i32, i32) = (
                rand::thread_rng().gen_range(0..self.arena_size.0),
                rand::thread_rng().gen_range(0..self.arena_size.1),
            );
            self.apple_pos = (test.0, test.1, true);
            for chunk in self.snake.iter() {
                if test.0 == chunk.0 && test.1 == chunk.1 {
                    self.apple_pos.2 = false;
                }
            }
            if self.apple_pos.2 {
                break;
            }
        }
    }

    fn update_state(&mut self) {
        self.state.reward = Fake::Val(self.reward_for_last_action);

        let head: (i32, i32);
        if let Some(thing) = self.snake.get(self.snake.len() - 1) {
            head = *thing;
        } else {
            panic!();
        }
        self.state.curr_apple.0 = self.apple_pos.0 - head.0;
        if self.state.curr_apple.0 > 0 {
            self.state.curr_apple.0 = 1;
        } else if self.state.curr_apple.0 < 0 {
            self.state.curr_apple.0 = -1;
        }
        self.state.curr_apple.1 = self.apple_pos.0 - head.0;
        if self.state.curr_apple.1 > 0 {
            self.state.curr_apple.1 = 1;
        } else if self.state.curr_apple.1 < 0 {
            self.state.curr_apple.1 = -1;
        }

        // populate self.state.map
        {
            self.state.map = Vec::with_capacity(self.bound);
            for i in 0..self.bound {
                self.state.map.push(Vec::with_capacity(self.bound));
                for _ in 0..self.bound {
                    self.state.map[i].push(MapState::Empty);
                }
            }
            let local_head = (self.bound as i32 / 2, self.bound as i32 / 2);
            for i in 0..self.bound as i32 {
                for j in 0..self.bound as i32 {
                    // if i != local_head.0 && j != local_head.1 {
                    //     self.state.map[i as usize][j as usize] = MapState::Empty;
                    //     continue;
                    // }
                    let test = (i - local_head.0 + head.0, j - local_head.1 + head.1);
                    for item in self.snake.iter() {
                        if item.0 == test.0 && item.1 == test.1 {
                            self.state.map[i as usize][j as usize] = MapState::Death;
                            break;
                        }
                    }
                    if test.0 < 0
                        || test.1 < 0
                        || test.0 >= self.arena_size.0
                        || test.1 >= self.arena_size.1
                    {
                        self.state.map[i as usize][j as usize] = MapState::Death;
                    }
                }
            }
        }
    }

    // returns true if the snake died this tick
    pub fn tick(&mut self, action: Action) -> bool {
        let mut new_head = (0, 0);
        if let Some(thing) = self.snake.get(self.snake.len() - 1) {
            new_head.0 = thing.0;
            new_head.1 = thing.1;
        } else {
            panic!();
        }

        match action {
            Action::YPos => {
                new_head.1 += 1;
            }
            Action::YNeg => {
                new_head.1 += -1;
            }
            Action::XPos => {
                new_head.0 += 1;
            }
            Action::XNeg => {
                new_head.0 += -1;
            }
        }

        let mut alive = true;
        alive = alive && new_head.0 >= 0 && new_head.1 >= 0;
        alive = alive && new_head.0 < self.arena_size.0 && new_head.1 < self.arena_size.1;

        for item in self.snake.iter() {
            alive = alive && !(item.0 == new_head.0 && item.1 == new_head.1);
        }

        if !alive {
            self.reset();
            self.reward_for_last_action = -4.0;
            self.update_state();
            self.render();
            return true;
        }

        self.snake.push(new_head);
        if new_head.0 == self.apple_pos.0 && new_head.1 == self.apple_pos.1 && self.apple_pos.2 {
            self.gen_apple();
            self.reward_for_last_action = 4.0;
        } else {
            self.snake.remove(0);
            self.reward_for_last_action = -0.01;
        }
        self.update_state();
        self.render();
        return false;
    }

    fn render(&self) {
        if !self.render {
            return;
        }
        if let Some(display) = &self.display {
            if let Some(program) = &self.program {
                let mut target = display.draw();

                target.clear_color(0.0, 0.0, 0.0, 1.0);

                let side_length = 1.0f32;
                let mut points: Vec<[f32; 2]> = Vec::new();

                for thing in self.snake.iter() {
                    points.push([thing.0 as f32, thing.1 as f32]);
                    points.push([thing.0 as f32, thing.1 as f32 + side_length]);
                    points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);

                    points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);
                    points.push([thing.0 as f32 + side_length, thing.1 as f32]);
                    points.push([thing.0 as f32, thing.1 as f32]);
                }
                let mut points_proper = points_to_points_proper(points, (0.0, 0.5, 0.0));

                if self.apple_pos.2 {
                    let mut points: Vec<[f32; 2]> = Vec::new();
                    points.push([self.apple_pos.0 as f32, self.apple_pos.1 as f32]);
                    points.push([
                        self.apple_pos.0 as f32,
                        self.apple_pos.1 as f32 + side_length,
                    ]);
                    points.push([
                        self.apple_pos.0 as f32 + side_length,
                        self.apple_pos.1 as f32 + side_length,
                    ]);

                    points.push([
                        self.apple_pos.0 as f32 + side_length,
                        self.apple_pos.1 as f32 + side_length,
                    ]);
                    points.push([
                        self.apple_pos.0 as f32 + side_length,
                        self.apple_pos.1 as f32,
                    ]);
                    points.push([self.apple_pos.0 as f32, self.apple_pos.1 as f32]);

                    points_proper.append(&mut points_to_points_proper(points, (1.0, 0.0, 0.0)));
                }

                let uniforms = uniform! {
                    matrix: self.transform_matrix.matrix,
                };

                let vertex_buffer = glium::VertexBuffer::new(display, &points_proper).unwrap();
                let index_buffer =
                    glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        program,
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();

                target.finish().unwrap();
            } else {
                panic!();
            }
        } else {
            panic!();
        }
    }
}

fn points_to_points_proper(points: Vec<[f32; 2]>, color: (f32, f32, f32)) -> Vec<Vertex> {
    let mut points_proper: Vec<Vertex> = Vec::new();
    for point in points {
        points_proper.push(Vertex {
            position: point,
            vec_color: color,
        });
    }
    return points_proper;
}
