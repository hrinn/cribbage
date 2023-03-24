use clap::Parser;
use cribbage::frame::Frame;
use cribbage::game::{Deck, Hand};
use cribbage::handle::Handle;
use std::io::{self, repeat};
use std::iter::{self, Cycle};
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
    score: u8,
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

    while players.len() < num_players {
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
                players.push(Player {
                    handle,
                    name,
                    score: 0,
                });
            }
            Ok(None) => println!("{} disconnected", addr),
            Ok(Some(_)) => println!("Incorrect first packet from {}", addr),
            Err(e) => println!("Bad first packet from {}: {}", addr, e),
        }
    }

    players
}

fn send_start(players: &mut Vec<Player>) -> Result<(), io::Error> {
    let names: Vec<String> = players.iter().map(|player| player.name.clone()).collect();

    let start_frame = Frame::Start(names);

    for player in players {
        player.handle.send_frame(&start_frame)?;
    }

    Ok(())
}

fn get_highest_score(players: &Vec<Player>) -> u8 {
    players.iter().map(|player| player.score).max().unwrap()
}

fn deal(deck: &mut Deck, players: &mut Vec<Player>, num_players: usize) -> Result<Hand, io::Error> {
    let num_deal = 8 - num_players; // 2 players get 6, 3 players get 5
    let mut crib = Hand::new();

    deck.shuffle();

    // Send each hand
    for player in players.iter_mut() {
        let hand = deck.deal(num_deal);
        println!("Dealing hand to {} ({})", player.name, hand);
        player.handle.send_frame(&Frame::Hand(hand))?;
    }

    // Get each discard
    for player in players.iter_mut() {
        let mut discard_hand = match player.handle.read_frame()? {
            Some(Frame::Hand(discard_hand)) => discard_hand,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };

        println!("Received discard from {} ({})", player.name, discard_hand);

        crib.combine(&mut discard_hand);
    }

    println!("Built crib!");

    // Draw magic card
    let magic = deck.draw_magic();
    println!("Drew magic card: {}", magic);
    let magic_frame = Frame::Card(magic);

    // Send magic card to clients
    for player in players.iter_mut() {
        player.handle.send_frame(&magic_frame)?;
    }

    // Wait for scores from nobs
    loop {}

    Ok(crib)
}

fn game_loop(players: &mut Vec<Player>, num_players: usize) -> Result<&Player, io::Error> {
    let mut deck = Deck::new();
    // let mut dealer_iter = players.iter().cycle();
    let indices: Vec<usize> = (1..=num_players).collect();
    let mut dealer_index = indices.into_iter().cycle().peekable();

    while get_highest_score(&players) < 121 {
        // let dealer = dealer_iter.next().unwrap();
        let dealer = players.get(dealer_index.next().unwrap()).unwrap();

        // Deal
        let crib = deal(&mut deck, players, num_players)?;

        // Play

        // Show
    }

    Ok(players.iter().max_by_key(|p| p.score).unwrap())
}

fn server(args: ServerArgs) -> Result<(), io::Error> {
    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr)?;

    let mut players = collect_players(listener, args.num_players);

    send_start(&mut players)?;

    let winner = game_loop(&mut players, args.num_players)?;

    Ok(())
}
