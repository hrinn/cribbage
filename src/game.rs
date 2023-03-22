use rand::{seq::SliceRandom, thread_rng};
use std::fmt;

#[derive(Copy, Clone)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

#[derive(Clone)]
pub struct Card {
    value: char,
    suit: Suit,
}

impl Card {
    pub fn to_short_name(&self) -> String {
        let mut name = String::new();

        name.push(self.value);

        match self.suit {
            Suit::Spades => name.push('S'),
            Suit::Hearts => name.push('H'),
            Suit::Diamonds => name.push('D'),
            Suit::Clubs => name.push('C'),
        }

        name
    }

    pub fn from_short_name(name: String) -> Card {
        let mut chars = name.chars();

        let value = chars.next().unwrap();

        let suit = match chars.next().unwrap() {
            'S' => Suit::Spades,
            'H' => Suit::Hearts,
            'D' => Suit::Diamonds,
            'C' => Suit::Clubs,
            _ => panic!("Invalid suit!"),
        };

        Card { value, suit }
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let mut cards = Vec::new();

        for suit in [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs] {
            for value in "A23456789TJQK".chars() {
                cards.push(Card { value, suit })
            }
        }

        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        assert!(
            self.cards.len() == 52,
            "Tried to shuffle with {} cards!",
            self.cards.len()
        );

        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn deal(&mut self, num: usize) -> Hand {
        Hand {
            cards: self.cards.split_off(self.cards.len() - num),
            magic: None,
        }
    }

    pub fn draw_magic(&self) -> Card {
        self.cards.last().unwrap().to_owned()
    }

    pub fn rejoin(&mut self, hand: &mut Hand) {
        self.cards.append(&mut hand.cards);
    }
}

pub struct Hand {
    cards: Vec<Card>,
    magic: Option<Card>,
}

impl Hand {
    pub fn from(cards: Vec<Card>) -> Hand {
        Hand { cards, magic: None }
    }

    pub fn new() -> Hand {
        Hand {
            cards: Vec::new(),
            magic: None,
        }
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn remove(&mut self, index: usize) -> Card {
        self.cards.remove(index)
    }

    pub fn combine(&mut self, other: &mut Hand) {
        while other.len() > 0 {
            self.push(other.remove(0))
        }
    }

    pub fn set_magic(&mut self, magic: Card) {
        self.magic = Some(magic);
    }

    pub fn score(&self) -> u8 {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    fn format_card(lines: &mut Vec<String>, card: &Card) {
        let second_line = lines.get(1).unwrap();

        let new = if card.value == 'T' {
            second_line.replacen("xz", "10", 1)
        } else {
            second_line.replacen("xz", &format!("{} ", card.value), 1)
        };

        lines.remove(1);
        lines.insert(1, new);

        let fourth_line = lines.get(3).unwrap();
        let suit = match card.suit {
            Suit::Spades => "♤",
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♧",
        };

        let new = fourth_line.replacen("y", suit, 1);
        lines.remove(3);
        lines.insert(3, new);
    }

    pub fn pretty_print(&self, index_flag: bool, magic_flag: bool) {
        let mut lines = vec![
            "┌─────┐ ".repeat(self.len()),
            "│xz   │ ".repeat(self.len()),
            "│     │ ".repeat(self.len()),
            "│    y│ ".repeat(self.len()),
            "└─────┘ ".repeat(self.len()),
        ];

        for card in &self.cards {
            Hand::format_card(&mut lines, card);
        }

        if index_flag {
            let mut index_line = String::new();

            for i in 0..self.len() {
                index_line.push_str(&format!("  ({})   ", i));
            }

            lines.push(index_line)
        }

        if magic_flag {
            if let Some(magic) = &self.magic {
                let mut magic_lines = vec![
                    String::from("| ┌─────┐"),
                    String::from("| │xz   │"),
                    String::from("| │  *  │"),
                    String::from("| │    y│"),
                    String::from("| └─────┘"),
                ];

                Hand::format_card(&mut magic_lines, magic);

                for i in 0..5 {
                    lines[i].push_str(magic_lines[i].as_str());
                }
            }
        }

        for line in lines {
            println!("{}", line);
        }
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();

        for card in &self.cards {
            s.push_str(&card.to_short_name());
            s.push(',');
        }

        write!(f, "{}", s)
    }
}
