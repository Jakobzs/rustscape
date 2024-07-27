use base64::{prelude::BASE64_STANDARD, Engine};
use rs2cache::{js5_masterindex::Js5MasterIndex, Cache};
use rscache::checksum;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;

pub async fn read_revision(
    revision: i32,
    socket: &mut TcpStream, /*, cache: std::sync::Arc<Cache>*/
) {
    let game_revision = socket.read_i32().await.unwrap();
    if game_revision != revision {
        return;
    }

    // Read the 4 arrrays which are encryption keys (prevent ppl accessing beta cache - Polar)
    for _ in 0..4 {
        socket.read_i32().await.unwrap();
    }

    socket.write_i8(0).await.unwrap();

    js5_loop(socket /* cache*/).await;
}

// Main loop for reading JS5 packets
async fn js5_loop(socket: &mut TcpStream /* , cache: std::sync::Arc<Cache>*/) {
    loop {
        let opcode = socket.read_u8().await;

        match opcode {
            Ok(opcode) => match opcode {
                0 | 1 => handle_file_request(socket /* , &cache*/).await,
                2 => handle_client_logged_in(socket).await,
                3 => handle_client_logged_out(socket).await,
                4 => handle_encryption_key_update(socket).await,
                _ => break,
            },
            Err(_) => break,
        }
    }
}

async fn handle_file_request(socket: &mut TcpStream /*cache: &std::sync::Arc<Cache>*/) {
    let index_id = socket.read_u8().await.unwrap();
    let archive_id = socket.read_u16().await.unwrap();

    // if requesting the meta index file
    if index_id == 255 && archive_id == 255 {
        // Decode base64 bytes

        /*
         let checksum = Checksum::new(&cache).unwrap();
         let encoded_checksum = checksum.encode().expect("failed encoding cache checksum");
        */

        let mut encoded_checksum = Vec::new();
        {
            let cache = Cache::open("cache").unwrap();
            let js5_masterindex = Js5MasterIndex::create(&cache.store);
            encoded_checksum = js5_masterindex.write();
        }

        // Buffer for the checksum
        let mut checksum_buf = Vec::new();
        checksum_buf.write_u8(index_id).await.unwrap();
        checksum_buf.write_u16(archive_id).await.unwrap();

        // Compression type
        checksum_buf.write_u8(0).await.unwrap();

        // Length of the checksum
        checksum_buf
            .write_u32(encoded_checksum.len() as u32)
            .await
            .unwrap();

        checksum_buf.write_all(&encoded_checksum).await.unwrap();

        socket.write_all(&checksum_buf).await.unwrap();
    }
    // if requesting a normal file
    else {
        /*
        let mut buf = cache
            .read(index_id, archive_id as u32)
            .expect("failed to read file from cache");

        // if index is not 255, we have to remove the useless version (2 bytes)
        if index_id != 255 {
            let len = buf.len();
            buf.truncate(len - 2);
        }

        let compression = *buf.get(0).unwrap();
        let length = u32::from_be_bytes([
            *buf.get(1).unwrap(),
            *buf.get(2).unwrap(),
            *buf.get(3).unwrap(),
            *buf.get(4).unwrap(),
        ]);

        buf.drain(..5);

        let mut data = vec![0; buf.len() + 8];
        data[0] = index_id;
        data[1..3].copy_from_slice(&(archive_id as u16).to_be_bytes());
        data[3] = compression;
        data[4..8].copy_from_slice(&length.to_be_bytes());
        data[8..].copy_from_slice(&buf);

        let chunks = data.len() / 512;
        for index_id in (0..data.len() + chunks).step_by(512) {
            if index_id == 0 || data.len() == 512 {
                continue;
            }

            data.insert(index_id, 255);
        }

        if let Err(e) = socket.write_all(&data).await {
            eprintln!("failed to write to socket; err = {:?}", e);
        }
        */
    }
}

async fn handle_client_logged_in(socket: &mut TcpStream) {
    socket.read_u8().await.unwrap();
    socket.read_u16().await.unwrap();
}

async fn handle_client_logged_out(socket: &mut TcpStream) {
    socket.read_u8().await.unwrap();
    socket.read_u16().await.unwrap();
}

async fn handle_encryption_key_update(socket: &mut TcpStream) {
    socket.read_u8().await.unwrap();
    socket.read_u16().await.unwrap();
}
