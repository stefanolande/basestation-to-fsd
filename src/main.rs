use std::error::Error;
use std::net::SocketAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{lookup_host, TcpListener, TcpStream};
use tokio::sync::watch::{channel, Receiver, Sender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const SBS_ADDRESS: &'static str = "vrs-hub.thehellnet.org:41009";
    let (tx, rx) = channel(String::new());

    tokio::spawn({
        let tx = tx.clone();
        async move {
            if let Err(e) = connect_to_feed(SBS_ADDRESS.to_string(), tx).await {
                eprintln!("Error connecting to feed: {:?}", e);
            }
        }
    });

    let listener = TcpListener::bind("127.0.0.1:1234").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let rx = rx.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, rx).await {
                eprintln!("Receiver disconnected: {:?}", e);
            }
        });
    }
}

async fn resolve(address: String) -> Result<SocketAddr, Box<dyn Error>> {
    let mut addrs = lookup_host(address).await?;
    if let Some(socket_addr) = addrs.next() {
        Ok(socket_addr)
    } else {
        Err("Could not resolve address".into())
    }
}

async fn connect_to_feed(
    sbs_address: String,
    sender: Sender<String>,
) -> Result<(), Box<dyn Error>> {
    let socket_addr = resolve(sbs_address).await?;

    let mut stream = TcpStream::connect(socket_addr).await?;
    let mut buffer_data = [0; 1024];

    loop {
        let n = stream.read(&mut buffer_data).await?;
        if n == 0 { break; }
        let message = String::from_utf8_lossy(&buffer_data[..n]).to_string();
        let _ = sender.send(message);
    }

    Ok(())
}

async fn handle_client(
    mut socket: TcpStream,
    mut receiver: Receiver<String>,
) -> Result<(), Box<dyn Error>> {
    println!("Client connected {:?}", socket.peer_addr()?);

    loop {
        receiver.changed().await?;

        let message = receiver.borrow_and_update().clone();
        if let Err(e) = socket.write_all(message.as_ref()).await {
            eprintln!("Error sending to client: {:?}", e);
            break;
        }
    }

    Ok(())
}
