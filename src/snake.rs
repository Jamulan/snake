use rand::Rng;

#[derive(Copy, Clone)]
pub enum Action {
    XPos,
    XNeg,
    YPos,
    YNeg,
}

pub struct Arena {
    // (x, y, distance_from_head)
    snake: Vec<(u32, u32, u32)>,
    // (x, y, is_spawned)
    apple_pos: (u32, u32, bool),
    arena_size: (u32, u32),
    length: u32,
}

impl Arena {
    pub fn new() -> Arena {
        let mut out = Arena {
            snake: Vec::new(),
            apple_pos: (0, 0, false),
            arena_size: (64, 64),
            length: 3,
        };
        out.snake.push((32, 32, 0));
        out.snake.push((32, 33, 1));
        out.snake.push((32, 34, 2));
        out.gen_apple();
        return out;
    }

    fn gen_apple(&mut self) {
        self.apple_pos.2 = false;
        loop {
            let test: (u32, u32) = (
                rand::thread_rng().gen_range(0..self.arena_size.0 + 1),
                rand::thread_rng().gen_range(0..self.arena_size.1 + 1),
            );
            for i in 0..self.snake.len() {
                let chunk = self.snake.get(i);
                let mut tmp: (u32, u32, u32);
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

    pub fn tick(&mut self, action: Action) {}
}
