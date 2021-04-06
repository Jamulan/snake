use rurel::strategy::learn::QLearning;
use snake_ai::{AiComponents, Config};

fn main() {
    let config = Config {
        bound: 3,
        arena_size: (16, 16),
        learning: QLearning::new(0.2, 0.01, 2.),
        render: false,
    };

    let mut ai = AiComponents::new(config);

    ai.train();
}
