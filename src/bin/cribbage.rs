use cribbage::handle::Handle;
use cribbage::frame::Frame;
use clap::Parser;
use std::net::TcpStream;

#[derive(Parser)]
struct ClientArgs {
    name: String,
    addr: String,
}

fn main() {
    let args = ClientArgs::parse();

    println!("Welcome {}", args.name);

    let stream = TcpStream::connect(args.addr);

    match stream {
        Ok(stream) => cribbage(stream, args.name),
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}

fn cribbage(stream: TcpStream, name: String) {
    println!("Connected to server!");

    let mut handle = Handle::new(stream);

    // Send name packet to server

    handle.send_frame(Frame::Name(name)).await;

    // Wait for game start packet, includes list of names
}
