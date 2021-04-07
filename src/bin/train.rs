use std::thread;

use rurel::strategy::learn::QLearning;

use snake_ai::{get_database, AiComponents, Config};

fn main() {
    let mut handles = vec![];
    let db = get_database();

    for i in 2..10 {
        if i % 2 == 0 {
            continue;
        }
        let db_real = db.clone();
        handles.push(thread::spawn(move || {
            let config = Config {
                bound: i,
                arena_size: (16, 16),
                learning: QLearning::new(0.1, 0.1, 2.),
                db: db_real,
            };
            let mut ai = AiComponents::new(config);

            ai.train();
        }));
    }
    {
        let local_db = db.clone();
        handles.push(thread::spawn(move || loop {
            thread::sleep(std::time::Duration::from_secs(60));
            let local_db_real = local_db.lock().unwrap();
            local_db_real.save().unwrap();
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    db.lock().unwrap().save().unwrap();
}
