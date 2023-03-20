use cribbage::handle::Handle;
use cribbage::frame::Frame;
use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncReadExt;

#[derive(Parser)]
struct ServerArgs {
    num_players: u8,
    #[arg(default_value_t = 31892)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = ServerArgs::parse();

    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        handle_client(socket).await;
    }
}

async fn handle_client(socket: TcpStream) {
    let mut handle = Handle::new(socket);

    match handle.read_frame().await {
        Frame::Name(name) => println!("{}", name),
        _ => panic!("Server received unexpected packet!"),
    }
}
