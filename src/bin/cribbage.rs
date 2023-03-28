use clap::Parser;
use cribbage::frame::Frame;
use cribbage::game::{is_run, Card, Hand};
use cribbage::handle::Handle;
use itertools::Itertools;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::io;
use std::net::TcpStream;

#[derive(Parser)]
struct ClientArgs {
    name: String,
    addr: String,
}

struct Player {
    name: String,
    score: u8,
}

struct Players {
    pub players: Vec<Player>,
    dealer_index: usize,
    player_index: usize,
}

impl Players {
    pub fn from(names: Vec<String>) -> Players {
        Players {
            players: names
                .into_iter()
                .map(|name| Player { name, score: 0 })
                .collect_vec(),
            dealer_index: 0,
            player_index: 0,
        }
    }

    pub fn print_scores(&self) {
        let min_score = self.players.iter().map(|p| p.score).min().unwrap() as isize;
        let max_score = self.max_score() as isize;

        let min_print = max(min_score - 5, 0);
        let max_print = min(max_score + 5, 121);

        //| ----- ---o- -----
        //| --o-- ----- -----

        for player in &self.players {
            for i in min_print..=max_print {
                if i == 0 {
                    if player.score == 0 {
                        print!("o ");
                    } else {
                        print!("| ");
                    }
                }

                if i == player.score as isize {
                    print!("o");
                } else {
                    print!("-");
                }

                if i % 5 == 0 {
                    print!(" ");
                }
            }

            println!(" ({}) {}", player.score, player.name);
        }
    }

    pub fn max_score(&self) -> u8 {
        self.players.iter().map(|p| p.score).max().unwrap()
    }

    pub fn player_with_max_score(&self) -> &Player {
        self.players
            .iter()
            .max_by_key(|p| p.score)
            .expect("No players found")
    }

    pub fn next_dealer(&mut self) -> String {
        let len = self.players.len();
        let dealer = self
            .players
            .get(self.dealer_index)
            .expect("No dealer found");
        self.dealer_index = (self.dealer_index + 1) % len;
        dealer.name.clone()
    }

    pub fn start_play(&mut self) {
        self.player_index = self.dealer_index;
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

    pub fn peek_player(&self) -> &Player {
        self.players
            .get(self.player_index)
            .expect("No player found")
    }

    pub fn decrement_player(&mut self) {
        self.player_index = (self.player_index + self.players.len() - 1) % self.players.len();
    }

    pub fn len(&self) -> usize {
        self.players.len()
    }
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

    let names = match handle.read_frame()? {
        Some(Frame::Start(names)) => names,
        _ => return Err(io::ErrorKind::InvalidData.into()),
    };

    println!("Game starting with players: {:?}", names);

    let players = Players::from(names);

    game_loop(&mut handle, players, args.name)?;

    Ok(())
}

fn game_loop(handle: &mut Handle, mut players: Players, name: String) -> Result<(), io::Error> {
    let num_players = players.len();

    while players.max_score() < 121 {
        let dealer = players.next_dealer();
        println!("Dealer: {}", dealer);

        let hand = get_hand(handle, num_players)?;

        play(handle, &hand, &mut players, &name)?;

        show(handle, hand, &mut players, &name, &dealer)?;

        println!("\nScores:");
        players.print_scores();
        std::thread::sleep(std::time::Duration::from_secs(10));
        println!("\n");
    }

    let winner = players.player_with_max_score();

    println!("{} wins!", winner.name);

    Ok(())
}

fn show(
    handle: &mut Handle,
    hand: Hand,
    players: &mut Players,
    name: &String,
    dealer: &String,
) -> Result<(), io::Error> {
    players.start_play();
    println!("\nShow!");

    for _ in 0..players.len() {
        let player = players.next_player();

        if &player.name == name {
            println!("\nYour Hand + Magic Card:");
            hand.pretty_print(false, true);
            player.score += hand.score();

            std::thread::sleep(std::time::Duration::from_secs(5));
            handle.send_frame(&Frame::Hand(hand.clone()))?;
        } else {
            // Receive hand from server and score
            println!("\n{}'s Hand + Magic Card:", player.name);

            std::thread::sleep(std::time::Duration::from_secs(5));
            let hand = match handle.read_frame()? {
                Some(Frame::Hand(hand)) => hand,
                _ => return Err(io::ErrorKind::InvalidData.into()),
            };

            hand.pretty_print(false, true);
            player.score += hand.score();
        }

        if &player.name == dealer {
            println!("\n{}'s Crib + Magic Card:", player.name);

            let crib = match handle.read_frame()? {
                Some(Frame::Hand(crib)) => crib,
                _ => return Err(io::ErrorKind::InvalidData.into()),
            };

            std::thread::sleep(std::time::Duration::from_secs(5));
            crib.pretty_print(false, true);
            player.score += crib.score();
        }
    }

    Ok(())
}

fn get_play(
    handle: &mut Handle,
    round_count: u8,
    name: &String,
    player: &String,
    playing_hand: &mut Hand,
) -> Result<(Option<Card>, bool), io::Error> {
    if player == name {
        if playing_hand.len() == 0 {
            println!("No cards left. Go!");
            handle.send_frame(&Frame::Play(None, true))?;
            return Ok((None, true));
        }

        let playable_hand = Hand::from(
            playing_hand
                .cards()
                .to_vec()
                .into_iter()
                .filter(|card| card.score_value() <= 31 - round_count)
                .collect_vec(),
            None,
        );

        if playable_hand.len() == 0 {
            println!("No playable cards. Go!");
            handle.send_frame(&Frame::Play(None, false))?;
            return Ok((None, false));
        }

        let card = prompt_user_play(playable_hand)?;
        println!("Playing: {}", card);
        playing_hand.remove_card(&card);
        handle.send_frame(&Frame::Play(Some(card.clone()), playing_hand.len() == 0))?;
        return Ok((Some(card), playing_hand.len() == 0));
    } else {
        // Wait for player
        println!("Waiting for {}...", player);

        match handle.read_frame()? {
            Some(Frame::Play(card, out)) => {
                if let Some(card) = &card {
                    println!("{} played {}", player, card);
                } else {
                    println!("{} couldn't play. Go!", player);
                }

                return Ok((card, out));
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
        let longest_run: Option<Vec<&Card>> = (0..history_len - 2)
            .collect::<Vec<usize>>()
            .into_iter()
            .map(|num_drop| {
                play_history
                    .iter()
                    .dropping(num_drop)
                    .sorted_by(|a, b| a.order().cmp(&b.order()))
                    .collect::<Vec<&Card>>()
            })
            .filter(|sorted_subset| is_run(&sorted_subset))
            .max_by(|a, b| a.len().cmp(&b.len()));

        if let Some(run) = longest_run {
            let run_length = run.len();
            score += run_length as u8;
            println!(
                "Run of {} for {}! ({})",
                run_length,
                score,
                run.iter().join(", ")
            );
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

fn players_finished(player_status: &HashMap<String, bool>) -> bool {
    player_status.values().all(|finished| *finished)
}

fn play(
    handle: &mut Handle,
    hand: &Hand,
    players: &mut Players,
    name: &String,
) -> Result<(), io::Error> {
    let mut playing_hand = Hand::from(hand.cards().to_vec(), None);
    let mut play_history: Vec<Card> = Vec::new();
    let mut round_count: u8 = 0;
    let mut go_count: usize = 0;
    let num_players = players.len();

    println!("\nPlay!");

    let mut player_status: HashMap<String, bool> = players
        .players
        .iter()
        .map(|player| (player.name.clone(), false))
        .collect();

    players.start_play();

    loop {
        while round_count < 31 {
            let mut player = players.next_player();
            println!("\nCount: {}", round_count);

            let (played_card, player_out) =
                get_play(handle, round_count, name, &player.name, &mut playing_hand)?;

            if let Some(card) = played_card {
                round_count += card.score_value();
                play_history.push(card);
                let score = score_play(&play_history);
                player.score += score;
                go_count = 0;
            } else {
                go_count += 1;
            }

            if go_count == num_players {
                println!("{} scored 1 for go!", player.name);
                player.score += 1;
                break;
            }

            if player_out {
                let player_entry = player_status
                    .get_mut(&player.name)
                    .expect("Player not found in player status map");
                if player_entry == &false {
                    *player_entry = true;
                }

                if players_finished(&player_status) {
                    println!("{} is last with cards! Go for 1!\n", player.name);
                    player.score += 1;

                    return Ok(());
                }
            }
        }

        println!("End of round!");
        if &players.peek_player().name == name {
            // Server expects a frame from me next
            handle.send_frame(&Frame::RoundDone)?;
        }

        play_history.clear();
        round_count = 0;
        go_count = 0;
        players.decrement_player();

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
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
