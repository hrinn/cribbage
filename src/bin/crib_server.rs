use clap::Parser;
use cribbage::frame::Frame;
use cribbage::handle::Handle;
use std::io;
use std::net::TcpListener;

#[derive(Parser)]
struct ServerArgs {
    num_players: usize,
    #[arg(default_value_t = 31892)]
    port: u16,
}

struct Player {
    handle: Handle,
    name: String,
}

fn main() {
    let args = ServerArgs::parse();

    if let Err(e) = server(args) {
        eprintln!("Error: {}", e);
    }
}

fn collect_players(listener: TcpListener, num_players: usize) -> Vec<Player> {
    let mut players: Vec<Player> = Vec::new();

    println!("Waiting for {} players...", num_players);

    while players.len() < num_players - 1 {
        let (stream, addr) = match listener.accept() {
            Ok((stream, addr)) => (stream, addr),
            Err(e) => {
                println!("Bad connection attempt: {}", e);
                continue;
            }
        };

        let mut handle = Handle::new(stream);

        match handle.read_frame() {
            Ok(Some(Frame::Name(name))) => {
                println!("Player {} connected from {}", name, addr);
                players.push(Player { handle, name });
            }
            Ok(None) => println!("{} disconnected", addr),
            Ok(Some(_)) => println!("Incorrect first packet from {}", addr),
            Err(e) => println!("Bad first packet from {}: {}", addr, e),
        }
    }

    players
}

fn send_start(players: Vec<Player>) -> Result<(), io::Error> {
    let names: Vec<String> = players.iter().map(|player| player.name.clone()).collect();

    let start_frame = Frame::Start(names);

    for mut player in players {
        player.handle.send_frame(&start_frame)?;
    }

    Ok(())
}

fn server(args: ServerArgs) -> Result<(), io::Error> {
    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr)?;

    let players = collect_players(listener, args.num_players);

    send_start(players)?;

    Ok(())
}
