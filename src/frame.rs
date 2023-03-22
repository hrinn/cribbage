use crate::game::Hand;

pub enum Frame {
    Name(String),       // Client sends name to server
    Start(Vec<String>), // Server tells client game starts, includes list of names
    Hand(Hand),         // Cribbage hand
}
