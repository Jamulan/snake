use rurel::strategy::learn::QLearning;
use snake_ai::{test, Config};

fn main() {
    let config = Config {
        bound: 3,
        arena_size: (16, 16),
        learning: QLearning::new(0.2, 0.1, 2.),
    };

    test(config);
}
