#[macro_use]
extern crate glium;
extern crate rust_lm;

mod snake;

use glium::{glutin, Surface};
use rust_lm::Mat4;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    vec_color: (f32, f32, f32),
}

//noinspection ALL
fn main() {
    // setup glium
    let events_loop = glium::glutin::event_loop::EventLoop::new();
    let wb = glium::glutin::window::WindowBuilder::new()
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Hello world");
    let cb = glium::glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

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

    let mut curr_action: snake::Action = snake::Action::XNeg;
    let mut game = snake::Arena::new();

    let transform_matrix = Mat4::identity()
        .scale_by(
            2.0 / (game.arena_size.0 as f32),
            2.0 / (game.arena_size.1 as f32),
            1.0,
        )
        .translate_by(-1.0, -1.0, 0.0);

    events_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                glutin::event::WindowEvent::KeyboardInput { input, .. } => match input.state {
                    glutin::event::ElementState::Pressed => match input.scancode {
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
                    glutin::event::ElementState::Released => match input.scancode {
                        _ => {}
                    },
                },
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let next_frame_time = std::time::Instant::now() + std::time::Duration::from_millis(32);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        game.tick(curr_action);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let side_length = 1.0f32;
        let mut points: Vec<[f32; 2]> = Vec::new();
        let tmp = game.snake.clone();

        for thing in tmp.iter() {
            points.push([thing.0 as f32, thing.1 as f32]);
            points.push([thing.0 as f32, thing.1 as f32 + side_length]);
            points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);

            points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);
            points.push([thing.0 as f32 + side_length, thing.1 as f32]);
            points.push([thing.0 as f32, thing.1 as f32]);
        }
        let mut points_proper = points_to_points_proper(points, (0.0, 0.5, 0.0));

        if game.apple_pos.2 {
            let mut points: Vec<[f32; 2]> = Vec::new();
            let thing = (game.apple_pos.0, game.apple_pos.1);
            points.push([thing.0 as f32, thing.1 as f32]);
            points.push([thing.0 as f32, thing.1 as f32 + side_length]);
            points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);

            points.push([thing.0 as f32 + side_length, thing.1 as f32 + side_length]);
            points.push([thing.0 as f32 + side_length, thing.1 as f32]);
            points.push([thing.0 as f32, thing.1 as f32]);

            points_proper.append(&mut points_to_points_proper(points, (1.0, 0.0, 0.0)));
        }

        let uniforms = uniform! {
            matrix: transform_matrix.matrix,
        };

        let vertex_buffer = glium::VertexBuffer::new(&display, &points_proper).unwrap();
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();

        // draw settled blocks

        target.finish().unwrap();
    });
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
