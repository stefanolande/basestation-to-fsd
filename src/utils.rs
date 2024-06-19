use std::net::SocketAddr;
use std::error::Error;
use tokio::net::lookup_host;

pub async fn resolve(address: String) -> Result<SocketAddr, Box<dyn Error>> {
    let mut addrs = lookup_host(address).await?;
    if let Some(socket_addr) = addrs.next() {
        Ok(socket_addr)
    } else {
        Err("Could not resolve address".into())
    }
}
