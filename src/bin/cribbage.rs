use clap::Parser;
use core::slice::Iter;
use cribbage::frame::Frame;
use cribbage::game::{Card, Hand};
use cribbage::handle::Handle;
use itertools::Itertools;
use std::io;
use std::iter::Cycle;
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
    handle.send_frame(&Frame::Name(args.name.clone()))?;

    // Wait for start packet
    println!("Waiting for players...");

    let players = match handle.read_frame()? {
        Some(Frame::Start(names)) => names,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    println!("Game starting with players: {:?}", players);

    game_loop(&mut handle, players, args.name)?;

    Ok(())
}

fn game_loop(handle: &mut Handle, players: Vec<String>, name: String) -> Result<(), io::Error> {
    let num_players = players.len();
    let mut dealer_iter = players.iter().cycle();

    loop {
        let dealer = dealer_iter.next().unwrap();
        println!("Dealer: {}", dealer);

        let hand = get_hand(handle, num_players)?;

        play(handle, hand, dealer_iter.clone(), &name, num_players)?
    }
}

fn get_play(
    handle: &mut Handle,
    round_count: u8,
    name: &String,
    player: &String,
    playing_hand: &mut Hand,
) -> Result<(Option<Card>, bool), io::Error> {
    if player == name {
        // Play
        if playing_hand.len() == 0 {
            println!("Out of cards. Go!");
            handle.send_frame(&Frame::GoEnd)?;
            return Ok((None, true));
        }

        let playable_hand = Hand::from(
            playing_hand
                .cards()
                .to_vec()
                .into_iter()
                .filter(|card| card.score_value() < 31 - round_count)
                .collect_vec(),
        );

        if playable_hand.len() == 0 {
            println!("No playable cards. Go!");
            handle.send_frame(&Frame::Go)?;
            return Ok((None, false));
        } else {
            let card = prompt_user_play(playable_hand)?;
            println!("Playing: {}", card);
            playing_hand.remove_card(&card);
            handle.send_frame(&Frame::Card(card.clone()))?;
            return Ok((Some(card), false));
        }
    } else {
        // Wait for player
        println!("Waiting for {}...", player);

        match handle.read_frame()? {
            Some(Frame::Card(card)) => {
                println!("{} played {}", player, card);
                return Ok((Some(card), false));
            }
            Some(Frame::Go) => {
                println!("{} couldn't play. Go!", player);
                return Ok((None, false));
            }
            Some(Frame::GoEnd) => {
                println!("{} is out of cards. Go!", player);
                return Ok((None, true));
            }
            _ => return Err(io::ErrorKind::InvalidData.into()),
        };
    }
}

fn score_play(play_history: &Vec<Card>) -> u8 {
    let mut score: u8 = 0;

    // Check if play is a run
    let history_len = play_history.len();
    if history_len > 2 {
        let run_length: usize = (0..history_len - 2)
            .collect::<Vec<usize>>()
            .into_iter()
            .map(|n| {
                (
                    play_history
                        .iter()
                        .dropping(n)
                        .map(|card| card.order())
                        .sorted()
                        .coalesce(|prev, curr| {
                            if prev + 1 == curr {
                                Ok(curr)
                            } else {
                                Err((prev, curr))
                            }
                        }),
                    n,
                )
            })
            .map(|(run, n)| if run.count() == 1 { history_len - n } else { 0 })
            .max()
            .unwrap_or(0);

        if run_length > 0 {
            score += run_length as u8;
            println!("Run of {} for {}!", run_length, score);
        }
    }

    // Check if play is a pair, triplet...
    let num_matching = play_history
        .iter()
        .rev()
        .skip(1)
        .take_while(|card| card.value == play_history.last().unwrap().value)
        .count()
        + 1;

    if num_matching >= 2 {
        score += (num_matching * (num_matching - 1)) as u8;

        match num_matching {
            2 => println!("Pair for {}!", score),
            3 => println!("Triplet for {}!!", score),
            4 => println!("Quadruplet for {}!!!", score),
            _ => panic!("More than four cards of the same rank in a play!"),
        }
    }

    // Check if play adds up to 15 (2 points)
    let sum = play_history
        .iter()
        .fold(0, |acc, card| acc + card.score_value());

    if sum == 15 {
        score += 2;
        println!("15 for {}!", score);
    }

    // Check if play adds up to 31 (2 points)
    if sum == 31 {
        score += 2;
        println!("31 for {}!", score);
    }

    score
}

fn play(
    handle: &mut Handle,
    hand: Hand,
    mut players: Cycle<Iter<String>>,
    name: &String,
    num_players: usize,
) -> Result<(), io::Error> {
    let mut playing_hand = Hand::from(hand.cards().to_vec());
    let mut play_history: Vec<Card> = Vec::new();
    let mut finished_players: usize = 0;
    let mut round_count: u8 = 0;
    let mut go_count: usize = 0;

    println!("\nPlay!");

    while finished_players < num_players {
        while round_count < 31 {
            let player = players.next().unwrap();
            println!("\nCount: {}", round_count);

            if go_count == num_players - 1 {
                println!("Go for 1!");
                // Add one to player's score
                break;
            }

            let (played_card, player_out) =
                get_play(handle, round_count, name, player, &mut playing_hand)?;

            if let Some(card) = played_card {
                round_count += card.score_value();
                play_history.push(card);
                let score = score_play(&play_history);
                // Add to players score
            } else {
                go_count += 1;
            }

            if player_out {
                finished_players += 1;
            }
        }

        play_history.clear();
        round_count = 0;
        go_count = 0;
    }

    Ok(())
}

fn prompt_user_play(mut playable_hand: Hand) -> Result<Card, io::Error> {
    let mut buf = String::new();

    loop {
        println!("Playable cards:");
        playable_hand.pretty_print(true, false);
        println!("Select a card to play: (i)");

        io::stdin().read_line(&mut buf)?;

        if let Ok(index) = buf.trim().parse() {
            if index < playable_hand.len() {
                let card = playable_hand.remove(index);
                return Ok(card);
            }
        }

        println!("Invalid input. Try again.");
        buf.clear();
    }
}

fn prompt_user_discard(num: usize, max_index: u8) -> Result<Vec<u8>, io::Error> {
    let mut buf = String::new();
    let mut indices: Vec<u8>;

    let prompt = if num == 2 {
        "Select two cards to discard: (i,j)"
    } else {
        "Select a card to discard: (i)"
    };

    loop {
        println!("{}", prompt);
        io::stdin().read_line(&mut buf)?;

        indices = buf.trim().split(',').flat_map(|s| s.parse().ok()).collect();

        if indices.len() == num && indices.iter().max().unwrap() < &max_index {
            indices.sort_by(|a, b| b.partial_cmp(a).unwrap());
            return Ok(indices);
        }

        println!("Invalid input. Try again.");
        buf.clear();
    }
}

fn get_hand(handle: &mut Handle, num_players: usize) -> Result<Hand, io::Error> {
    // Wait for hand
    let mut hand = match handle.read_frame()? {
        Some(Frame::Hand(hand)) => hand,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    println!("\nHand:");
    hand.pretty_print(true, false);

    let num_discard = 4 - num_players;

    let discard = prompt_user_discard(num_discard, hand.len().try_into().unwrap())?;

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

    println!("Magic card dealt! ({})", magic);
    hand.set_magic(magic);

    Ok(hand)
}
