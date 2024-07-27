use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn read_revision(
    revision: i32,
    socket: &mut TcpStream, /*, cache: std::sync::Arc<Cache>*/
) {
    let game_revision = socket.read_i32().await.unwrap();
    if game_revision != revision {
        return;
    }

    socket.write_i8(0).await.unwrap();

    //js5_loop(socket, cache).await;
}
