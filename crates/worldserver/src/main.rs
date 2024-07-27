use crate::{input::input, render::render, setup::setup, update::update};
use anyhow::Result;
use crossbeam::queue::SegQueue;
use mlua::Lua;
use network::setup_login_acceptor;
use std::{
    cmp,
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};
use tokio::{io::AsyncReadExt, net::TcpListener};
use tracing::info;

mod config;
mod input;
mod network;
mod render;
mod setup;
mod update;

const TICK_RATE: i128 = 600;

pub struct LoginRequest {
    //auth_player_data: AuthenticatedPlayerData,
    //client_packet_queue: Arc<SegQueue<ClientPacket>>,
    //server_packet_queue: Arc<SegQueue<ServerPacket>>,
    rw_lock_something: Arc<RwLock<bool>>,
}

// The main thread is considered the game thread. Therefore, main is not async
fn main() -> Result<()> {
    let (mut lua, login_queue, _guard1, _guard2) = setup(223)?;

    // Create thread that spawns the tokio runtime and accepts connections
    setup_login_acceptor(223, &login_queue);

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

        info!("Tick took: {}ms", elapsed_time.as_millis());

        // Sleep until the next tick
        thread::sleep(Duration::from_millis(sleep_time));
    }

    Ok(())
}

pub struct Player {
    name: String,
}

pub struct World {
    name: String,
    players: Vec<Player>,
    should_shutdown: bool,
}
