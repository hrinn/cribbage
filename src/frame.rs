use crate::game::{Card, Hand};

pub enum Frame {
    Name(String),             // Client sends name to server
    Start(Vec<String>),       // Server tells client game starts, includes list of names
    Hand(Hand),               // Cribbage hand
    Card(Card),               // Single card
    Play(Option<Card>, bool), // A single move (card played, is player out of cards)
    RoundDone,                // Client tells server a round is done
}
