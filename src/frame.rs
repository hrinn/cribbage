use crate::game::{Card, Hand};

pub enum Frame {
    Name(String),       // Client sends name to server
    Start(Vec<String>), // Server tells client game starts, includes list of names
    Hand(Hand),         // Cribbage hand
    Card(Card),         // Single card
    Points(u8),         // Points
}
