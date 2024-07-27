use crate::LoginRequest;
use crossbeam::queue::SegQueue;
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream};

struct LoginSocket {
    socket: TcpStream,
    session_id: u64,
}

impl LoginSocket {
    fn new(socket: TcpStream, session_id: u64) -> Self {
        Self { socket, session_id }
    }
}

pub async fn start_login_socket(
    revision: i32,
    socket: TcpStream,
    //cache: Arc<Cache>,
    //config: &Arc<Config>,
    login_queue: &Arc<SegQueue<LoginRequest>>,
    //world_player_status: &Arc<Mutex<WorldPlayerStatus>>,
) {
    let mut login_socket = LoginSocket::new(socket, 0);

    login_socket.socket.write_u8(0).await.unwrap();
    login_socket
        .socket
        .write_u64(login_socket.session_id)
        .await
        .unwrap(); // session id

    /*
        read_login_packet_opcode(
            revision,
            login_socket,
            cache,
            config,
            login_queue,
            world_player_status,
        )
        .await;
    */
}
