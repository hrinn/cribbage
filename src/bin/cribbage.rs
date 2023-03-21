use clap::Parser;
use cribbage::frame::Frame;
use cribbage::handle::Handle;
use std::io;
use std::net::TcpStream;

#[derive(Parser)]
struct ClientArgs {
    name: String,
    addr: String,
}

fn main() {
    let args = ClientArgs::parse();

    if let Err(e) = cribbage(args) {
        eprintln!("Error: {}", e);
    }
}

fn cribbage(args: ClientArgs) -> Result<(), io::Error> {
    println!("Welcome {}", args.name);

    let mut handle = Handle::new(TcpStream::connect(args.addr)?);

    println!("Connected to server!");

    // Send name packet to server
    handle.send_frame(&Frame::Name(args.name))?;

    // Wait for start packet
    println!("Waiting for players...");

    if let Some(Frame::Start(names)) = handle.read_frame()? {
        println!("Game starting with players: {:?}", names);
    } else {
        return Err(io::ErrorKind::InvalidData.into());
    }

    Ok(())
}
