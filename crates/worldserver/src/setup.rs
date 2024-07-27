use crate::{config::Config, LoginRequest, Player, World};
use anyhow::Result;
use crossbeam::queue::SegQueue;
use mlua::Lua;
use std::sync::Arc;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

pub fn setup(
    revision: i32,
) -> Result<(Lua, Arc<SegQueue<LoginRequest>>, WorkerGuard, WorkerGuard)> {
    let (_guard1, _guard2) = setup_logging()?;

    let world = World {
        name: "World".to_string(),
        players: vec![Player {
            name: "Player".to_string(),
        }],
        should_shutdown: false,
    };

    let login_queue: Arc<SegQueue<LoginRequest>> = Arc::new(SegQueue::new());

    let lua = Lua::new();
    lua.set_app_data(world);
    Ok((lua, login_queue, _guard1, _guard2))
}

fn setup_logging() -> Result<(WorkerGuard, WorkerGuard)> {
    // Create file appender
    let file_appender = tracing_appender::rolling::hourly("./logs", "worldserver.log");

    // Create non-blocking appenders
    let (console_non_blocking, console_non_blocking_guard) =
        tracing_appender::non_blocking(std::io::stdout());
    let (file_non_blocking, file_non_blocking_guard) =
        tracing_appender::non_blocking(file_appender);

    // Create the subscriber
    let subscriber = Registry::default()
        .with(
            fmt::Layer::default()
                .with_writer(file_non_blocking)
                .with_ansi(false),
        )
        .with(fmt::Layer::default().with_writer(console_non_blocking));

    // Set the global subscriber
    tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");

    info!("Logging initialized");

    Ok((console_non_blocking_guard, file_non_blocking_guard))
}

fn setup_config() -> Result<Arc<Config>> {
    info!("Loading config...");
    let config_file_string =
        std::fs::read_to_string("worldserver.toml").expect("failed opening worldserver.toml");
    let config: Arc<Config> = Arc::new(toml::from_str(&config_file_string)?);
    info!("Finished loading config");
    Ok(config)
}
