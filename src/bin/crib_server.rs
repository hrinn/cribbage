use clap::Parser;
use cribbage::frame::Frame;
use cribbage::handle::Handle;
use std::io;
use std::net::TcpListener;

#[derive(Parser)]
struct ServerArgs {
    num_players: u8,
    #[arg(default_value_t = 31892)]
    port: u16,
}

struct Player {
    handle: Handle,
}

fn main() {
    let args = ServerArgs::parse();

    if let Err(e) = server(args) {
        eprintln!("Error: {}", e);
    }
}

fn server(args: ServerArgs) -> Result<(), io::Error> {
    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr)?;

    let mut connected_players: u8 = 0;

    println!("Waiting for {} players...", args.num_players);

    let mut players: Vec<Player> = Vec::new();

    while connected_players < args.num_players {
        let (socket, addr) = listener.accept()?;
        let mut handle = Handle::new(socket);

        if let Some(Frame::Name(name)) = handle.read_frame()? {
            println!("Player {} connected from {}", name, addr);
            connected_players += 1;
            players.push(Player { handle });
        } else {
            eprintln!("Bad connection from {}", addr);
        }
    }

    Ok(())
}
