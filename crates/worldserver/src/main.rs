use crate::{input::input, render::render, setup::setup, update::update};
use mlua::Lua;
use std::{
    cmp, thread,
    time::{Duration, Instant},
};

mod config;
mod input;
mod render;
mod setup;
mod update;

const TICK_RATE: i128 = 600;

// The main thread is considered the game thread. Therefore main is not async
fn main() {
    let world = setup();

    // Create mlua lua state
    let lua = Lua::new();
    lua.set_app_data(world);

    // Create thread that spawns the tokio runtime and accepts connections
    thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Run the game loop
            init_tokio().await;
        });
    });

    loop {
        let start_time = Instant::now();

        input(&lua);
        update(&lua);
        render(&lua);

        let elapsed_time = start_time.elapsed();
        let sleep_time = cmp::max(TICK_RATE - elapsed_time.as_millis() as i128, 0) as u64;

        if lua.app_data_ref::<World>().unwrap().should_shutdown {
            break;
        }

        println!("Tick took: {}ms", elapsed_time.as_millis());

        // Sleep until the next tick
        thread::sleep(Duration::from_millis(sleep_time));
    }
}

pub struct Player {
    name: String,
}

pub struct World {
    name: String,
    players: Vec<Player>,
    should_shutdown: bool,
}

async fn init_tokio() {
    println!("Tokio here");
}
