use rurel::strategy::learn::QLearning;
use snake_ai::{AiComponents, Config};
use std::thread;

fn main() {
    for i in 2..9 {
        if i % 2 == 0 {
            continue;
        }
        thread::spawn(move || {
            let config = Config {
                bound: i,
                arena_size: (16, 16),
                learning: QLearning::new(0.1, 0.1, 2.),
                render: false,
            };

            let mut ai = AiComponents::new(config);

            ai.train();
        });
        thread::sleep(std::time::Duration::from_secs(10));
    }
    loop {}
}
