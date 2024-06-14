use std::error::Error;
use std::net::SocketAddr;

use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{lookup_host, TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    const SBS_ADDRESS: &'static str = "vrs-hub.thehellnet.org:41009";
    println!("bus initialized");

    let (sender, receiver) = unbounded();

    tokio::spawn(async move {
        if let Err(e) = connect_to_feed(SBS_ADDRESS.to_string(), sender).await {
            eprintln!("Error connecting to feed: {:?}", e);
        }
    });

    let listener = TcpListener::bind("127.0.0.1:1234").await?;

    loop {
        let r = receiver.clone();
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, r).await {
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
    let mut buffer = [0; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 {
            break; // Connection closed
        }
        let message = String::from_utf8_lossy(&buffer[..n]).to_string();
        sender.send(message)?
    }

    Ok(())
}

async fn handle_client(
    mut socket: TcpStream,
    receiver: Receiver<String>,
) -> Result<(), Box<dyn Error>> {
    println!("client connected {:?}", socket.peer_addr()?);
    loop {
        match receiver.recv() {
            Ok(message) => {
                socket.write_all(message.as_ref()).await?;
            }
            Err(e) => {
                eprintln!("Error receiving from bus: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}
