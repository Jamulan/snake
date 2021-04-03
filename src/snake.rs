use glium::{glutin, Surface};
use rand::Rng;
use rust_lm::Mat4;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
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
    pub(crate) snake: Vec<(i32, i32)>,
    // (x, y, is_spawned)
    pub(crate) apple_pos: (i32, i32, bool),
    pub(crate) arena_size: (i32, i32),
    display: glium::Display,
    program: glium::Program,
    transform_matrix: Mat4,
}

impl Arena {
    pub fn new(events_loop: &glutin::event_loop::EventLoop<()>, arena_size: (i32, i32)) -> Arena {
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
            display: display,
            program: program,
            transform_matrix: transform_matrix,
        };
        out.reset();
        return out;
    }

    fn new_snake(&mut self) {
        let mut new_snake = Vec::new();
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 2));
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 1));
        new_snake.push((self.arena_size.0 / 2, (self.arena_size.1 / 2) - 0));
        self.snake = new_snake;
    }

    fn reset(&mut self) {
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
            for i in 0..self.snake.len() {
                let chunk = self.snake.get(i);
                let tmp: (i32, i32);
                if let Some(thing) = chunk {
                    tmp = *thing;
                } else {
                    break;
                }
                let chunk = tmp;
                if test.0 != chunk.0 && test.1 != chunk.1 {
                    self.apple_pos = (test.0, test.1, true);
                    break;
                }
            }
            if self.apple_pos.2 {
                break;
            }
        }
    }

    // returns reward for tick
    pub fn tick(&mut self, action: Action) -> f64 {
        let mut new_head = (0, 0);
        if let Some(thing) = self.snake.get(self.snake.len() - 1) {
            new_head.0 = thing.0;
            new_head.1 = thing.1;
        } else {
            panic!();
        }
        let old_head = new_head.clone();

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
            return -4.0;
        }

        self.snake.push(new_head);
        if new_head.0 == self.apple_pos.0 && new_head.1 == self.apple_pos.1 && self.apple_pos.2 {
            self.gen_apple();
            return 4.0;
        } else {
            self.snake.remove(0);
            // let new_dist = (new_head.0 - self.apple_pos.0, new_head.1 - self.apple_pos.1);
            // let old_dist = (old_head.0 - self.apple_pos.0, old_head.1 - self.apple_pos.1);
            // if new_dist.0.abs() < old_dist.0.abs() || new_dist.1.abs() < old_dist.1.abs() {
            //     return 1.0;
            // } else if new_dist.0.abs() > old_dist.0.abs() || new_dist.1.abs() > old_dist.1.abs() {
            //     return -1.0;
            // } else {
            //     return 0.0;
            // }
            return -0.01;
        }
    }

    pub fn render(&self) {
        let mut target = self.display.draw();

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

        let vertex_buffer = glium::VertexBuffer::new(&self.display, &points_proper).unwrap();
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        target.finish().unwrap();
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
