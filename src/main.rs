use std::error::Error;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch::{channel, Receiver, Sender};

use fsd::FSDMessage;
mod utils;
mod fsd;

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

    let listener = TcpListener::bind("127.0.0.1:6809").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        let rx = rx.clone();

        tokio::spawn(async {
            if let Err(e) = handle_client(socket, rx).await {
                eprintln!("Receiver disconnected: {:?}", e);
            }
        });
    }
}

async fn connect_to_feed(
    sbs_address: String,
    sender: Sender<String>,
) -> Result<(), Box<dyn Error>> {
    let socket_addr = utils::resolve(sbs_address).await?;

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

    let mut buffer_data = [0; 1024];
    let n = socket.read(&mut buffer_data).await?;
    let message = FSDMessage::from_string(String::from_utf8_lossy(&buffer_data[..n]).to_string())?;
    println!("{:?}", message);
    match message {
        FSDMessage::LoginRequest(login_request) => {
            socket.write_all(FSDMessage::LoginResponse(login_request.to_response()).to_bytes().as_slice()).await?;

            loop {
                receiver.changed().await?;

                let message = receiver.borrow_and_update().clone();
                if let Err(e) = socket.write_all(message.as_ref()).await {
                    eprintln!("Error sending to client: {:?}", e);
                    break;
                }
            }
        }
        _ => {}
    }


    Ok(())
}
