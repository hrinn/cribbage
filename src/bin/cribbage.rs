use clap::Parser;
use cribbage::frame::Frame;
use cribbage::game::{Card, Hand};
use cribbage::handle::Handle;
use itertools::Itertools;
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

fn prompt_user_cards(prompt: &str, num_cards: usize, max_cards: u8) -> Result<Vec<u8>, io::Error> {
    let mut buf = String::new();
    let mut cards: Vec<u8>;

    loop {
        println!("{}", prompt);
        io::stdin().read_line(&mut buf)?;

        cards = buf.trim().split(',').flat_map(|s| s.parse().ok()).collect();

        if cards.len() == num_cards && cards.iter().max().unwrap() < &max_cards {
            cards.sort_by(|a, b| b.partial_cmp(a).unwrap());
            return Ok(cards);
        }

        println!("Invalid input. Try again.");
        buf.clear();
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

    let players = match handle.read_frame()? {
        Some(Frame::Start(names)) => names,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    println!("Game starting with players: {:?}", players);

    game_loop(&mut handle, players)?;

    Ok(())
}

fn game_loop(handle: &mut Handle, players: Vec<String>) -> Result<(), io::Error> {

    let num_players = players.len();
    let mut dealer_iter = players.iter().cycle().peekable();

    for dealer in dealer_iter {
        println!("Dealer: {}", dealer);

        let hand = get_hand(handle, num_players)?;

    }

    Ok(())
}

fn get_hand(handle: &mut Handle, num_players: usize) -> Result<Hand, io::Error> {
    // Wait for hand
    let mut hand = match handle.read_frame()? {
        Some(Frame::Hand(hand)) => hand,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    println!("\nHand:");
    hand.pretty_print(true, false);

    let prompt = if num_players == 2 {
        "Select two cards to discard: (i,j)"
    } else {
        "Select a card to discard: (i)"
    };

    let num_discard = 4 - num_players;

    let discard = prompt_user_cards(prompt, num_discard, hand.len().try_into().unwrap())?;

    let mut discard_hand = Hand::new();

    // Remove discarded cards from hand
    for i in discard {
        discard_hand.push(hand.remove(i.into()));
    }

    println!("Discarding... ({})", discard_hand.cards().iter().join(", "));

    // Send discard to server
    handle.send_frame(&Frame::Hand(discard_hand))?;

    // Wait for magic card
    let magic = match handle.read_frame()? {
        Some(Frame::Card(magic)) => magic,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    hand.set_magic(magic);
    println!("Magic card dealt!");

    println!("\nFinal Hand + Magic Card:");
    hand.pretty_print(false, true);

    Ok(hand)
}
