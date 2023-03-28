use clap::Parser;
use cribbage::frame::Frame;
use cribbage::game::{Deck, Hand};
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
    finished: bool,
}

struct Players {
    players: Vec<Player>,
    dealer_index: usize,
    player_index: usize,
}

impl Players {
    pub fn from(players: Vec<Player>) -> Players {
        Players {
            players,
            dealer_index: 0,
            player_index: 0,
        }
    }

    pub fn next_dealer(&mut self) -> &mut Player {
        let len = self.players.len();
        let dealer = self
            .players
            .get_mut(self.dealer_index)
            .expect("No dealer found");
        self.dealer_index = (self.dealer_index + 1) % len;
        dealer
    }

    pub fn start_play(&mut self) {
        self.player_index = self.dealer_index;
        for player in self.players.iter_mut() {
            player.finished = false;
        }
    }

    pub fn next_player(&mut self) -> &mut Player {
        let len = self.players.len();
        let player = self
            .players
            .get_mut(self.player_index)
            .expect("No dealer found");
        self.player_index = (self.player_index + 1) % len;
        player
    }

    pub fn decrement_player(&mut self) {
        let len = self.players.len();
        self.player_index = (self.player_index + len - 1) % len;
    }

    pub fn players_finished(&self) -> bool {
        self.players.iter().all(|player| player.finished)
    }
}

fn main() {
    let args = ServerArgs::parse();

    if let Err(e) = server(args) {
        eprintln!("Error: {}", e);
    }
}

fn collect_players(listener: TcpListener, num_players: usize) -> Players {
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
                    finished: false,
                });
            }
            Ok(None) => println!("{} disconnected", addr),
            Ok(Some(_)) => println!("Incorrect first packet from {}", addr),
            Err(e) => println!("Bad first packet from {}: {}", addr, e),
        }
    }

    Players::from(players)
}

fn send_start(players: &mut Players) -> Result<(), io::Error> {
    let names: Vec<String> = players
        .players
        .iter()
        .map(|player| player.name.clone())
        .collect();

    let start_frame = Frame::Start(names);

    for player in &mut players.players {
        player.handle.send_frame(&start_frame)?;
    }

    Ok(())
}

fn deal(deck: &mut Deck, players: &mut Players, num_players: usize) -> Result<Hand, io::Error> {
    let num_deal = 8 - num_players; // 2 players get 6, 3 players get 5
    let mut crib = Hand::new();

    println!("Shuffling deck...");
    deck.shuffle();

    // Send each hand
    for player in players.players.iter_mut() {
        let hand = deck.deal(num_deal);
        println!("Dealing hand to {} ({})", player.name, hand);
        player.handle.send_frame(&Frame::Hand(hand))?;
    }

    // Get each discard
    for player in players.players.iter_mut() {
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
    crib.set_magic(magic.clone());
    let magic_frame = Frame::Card(magic.to_owned());

    // Send magic card to clients
    for player in players.players.iter_mut() {
        player.handle.send_frame(&magic_frame)?;
    }

    Ok(crib)
}

fn get_play(player: &mut Player) -> Result<Frame, io::Error> {
    println!("Waiting for play from {}", player.name);

    let frame = match player.handle.read_frame()? {
        Some(Frame::Play(card, out)) => {
            if out {
                player.finished = true;
            }
            Frame::Play(card, out)
        }
        Some(Frame::RoundDone) => Frame::RoundDone,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };
    Ok(frame)
}

fn play(players: &mut Players) -> Result<(), io::Error> {
    players.start_play();
    println!("Starting play");

    while !players.players_finished() {
        let player = players.next_player();

        let frame = if player.finished {
            Frame::Play(None, true)
        } else {
            get_play(player)?
        };

        let name = player.name.clone();

        if let Frame::RoundDone = frame {
            println!("Client notified server round is done.");
            players.decrement_player();
            players.decrement_player();
        } else {
            forward_frame(players, frame, &name)?;
        }
    }

    Ok(())
}

fn forward_frame(players: &mut Players, frame: Frame, name: &String) -> Result<(), io::Error> {
    for player in &mut players.players {
        if name == &player.name {
            continue;
        }

        player.handle.send_frame(&frame)?;
    }

    Ok(())
}

fn show(players: &mut Players, crib: Hand) -> Result<Vec<Hand>, io::Error> {
    let mut hands = Vec::new();
    players.start_play();
    println!("Starting show");

    for _ in 0..players.players.len() {
        let player = players.next_player();

        println!("Waiting for hand from {}...", player.name);
        let hand = match player.handle.read_frame()? {
            Some(Frame::Hand(hand)) => hand,
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };

        let name = player.name.clone();
        drop(player);

        // Send hand to other players
        println!("Forwarding hand to other players...");
        forward_frame(players, Frame::Hand(hand.clone()), &name)?;

        // Add hand to list
        hands.push(hand);
    }

    println!("Broadcasting crib...");
    let crib_frame = Frame::Hand(crib);

    for player in &mut players.players {
        player.handle.send_frame(&crib_frame)?;
    }

    Ok(hands)
}

fn game_loop(players: &mut Players, num_players: usize) -> Result<(), io::Error> {
    let mut deck = Deck::new();

    loop {
        let dealer = players.next_dealer();
        println!("Dealer = {}", dealer.name);

        // Deal
        let crib = deal(&mut deck, players, num_players)?;

        // Play
        play(players)?;

        // Show
        let hands = show(players, crib.clone())?;

        // Recover deck
        for hand in hands {
            println!("Recovered hand: {}", hand);
            deck.rejoin(hand);
        }

        println!("Recovered crib: {}", crib);
        deck.rejoin(crib);
    }
}

fn server(args: ServerArgs) -> Result<(), io::Error> {
    let addr = format!("0.0.0.0:{}", args.port);

    println!("Launching server on {}", addr);

    let listener = TcpListener::bind(addr)?;

    let mut players = collect_players(listener, args.num_players);

    send_start(&mut players)?;

    game_loop(&mut players, args.num_players)?;

    Ok(())
}
