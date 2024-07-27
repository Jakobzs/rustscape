use crate::LoginRequest;
use anyhow::Result;
use axum::{routing::get, Router};
use crossbeam::queue::SegQueue;
use rs2cache::Cache;
use std::{fs, net::SocketAddr, sync::Arc, thread};
use tokio::{io::AsyncReadExt, net::TcpListener, sync::Mutex};

mod js5;
mod login;

const LOGIN_SERVICE: u8 = 14;
const JS5_SERVICE: u8 = 15;

pub fn setup_login_acceptor(
    revision: i32,
    cache: Arc<Mutex<Cache>>,
    //config: &Arc<Config>,
    login_queue: &Arc<SegQueue<LoginRequest>>,
    //world_player_status: &Arc<Mutex<WorldPlayerStatus>>,
) -> Result<()> {
    let login_queue = login_queue.clone();
    //let config = config.clone();
    //let world_player_status = world_player_status.clone();

    thread::spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            setup_ws_server().unwrap();

            accept_login_sockets(revision, cache, &login_queue)
                .await
                .unwrap();
        });
    });

    Ok(())
}

async fn accept_login_sockets(
    revision: i32,
    cache: Arc<Mutex<Cache>>,
    //config: Arc<Config>,
    login_queue: &Arc<SegQueue<LoginRequest>>,
    //world_player_status: &Arc<Mutex<WorldPlayerStatus>>,
) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:43594").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let cache = cache.clone();
        //let mut config = config.clone();
        let login_queue = login_queue.clone();
        //let world_player_status = world_player_status.clone();

        tokio::spawn(async move {
            match socket.set_nodelay(true) {
                Ok(n) => n,
                Err(e) => eprintln!("Failed to set nodelay on socket, error: {}", e),
            }

            if let Ok(service_selection) = socket.read_u8().await {
                match service_selection {
                    LOGIN_SERVICE => {
                        login::start_login_socket(
                            revision,
                            socket,
                            //cache,
                            //&mut config,
                            &login_queue,
                            //&world_player_status,
                        )
                        .await
                    }
                    JS5_SERVICE => js5::read_revision(revision, &mut socket, cache).await,
                    _ => (),
                }
            }
        });
    }
}

fn setup_ws_server() -> Result<()> {
    tokio::spawn(async {
        let app = Router::new()
            .route("/jav_config.ws", get(jav_config))
            .route("/world_list.ws", get(world_list));

        let addr = SocketAddr::from(([127, 0, 0, 1], 80));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    Ok(())
}

async fn jav_config() -> String {
    fs::read_to_string("jav_config.ws").expect("failed reading jav_config.ws")
}

async fn world_list() -> String {
    fs::read_to_string("world_list.ws").expect("failed reading world_list.ws")
}
