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
    pub(crate) snake: Vec<(i32, i32)>,
    // (x, y, is_spawned)
    pub(crate) apple_pos: (i32, i32, bool),
    pub(crate) arena_size: (i32, i32),
}

impl Arena {
    pub fn new() -> Arena {
        let mut out = Arena {
            snake: Vec::new(),
            apple_pos: (0, 0, false),
            arena_size: (64, 64),
        };
        out.snake.push((32, 32));
        out.snake.push((32, 33));
        out.snake.push((32, 34));
        out.gen_apple();
        return out;
    }

    fn reset(&mut self) {
        let new_self = Arena::new();
        self.snake = new_self.snake;
        self.apple_pos = new_self.apple_pos;
        self.arena_size = new_self.arena_size;
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

    pub fn tick(&mut self, action: Action) {
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
            return;
        }

        self.snake.push(new_head);
        if new_head.0 == self.apple_pos.0 && new_head.1 == self.apple_pos.1 && self.apple_pos.2 {
            self.gen_apple();
        } else {
            self.snake.remove(0);
        }
    }
}
