mod cli;
mod networking;

use std::net::{IpAddr, SocketAddr};
use cli::{ServerArgs};


#[tokio::main]
async fn main() {
    let args = ServerArgs::parse_program_args();
    let socket_address = SocketAddr::new(IpAddr::V4(args.host), args.port);

    networking::start(socket_address).await;
}
