use clap::Parser;
use cribbage::frame::Frame;
use cribbage::handle::Handle;
use std::net::TcpListener;

#[derive(Parser)]
struct ServerArgs {
    num_players: u8,
    #[arg(default_value_t = 31892)]
    port: u16,
}

fn main() {
    let args = ServerArgs::parse();

    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr).unwrap();

    let mut connected_players: u8 = 0;

    while connected_players < args.num_players {
        let (socket, _) = listener.accept().unwrap();
        connected_players += 1;

        let mut handle = Handle::new(socket);
        match handle.read_frame() {
            Ok(Some(Frame::Name(name))) => println!("Player {} connected!", name),
            Ok(None) => panic!("no packet from client"),
            Ok(Some(_)) => panic!("unexpected packet from client"),
            Err(e) => panic!("error reading from client: {}", e),
        }
    }
}
