use itertools::Itertools;
use rand::{seq::SliceRandom, thread_rng};
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Card {
    pub value: char,
    pub suit: Suit,
}

impl Card {
    pub fn score_value(&self) -> u8 {
        match self.value {
            'A' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'T' => 10,
            'J' => 10,
            'Q' => 10,
            'K' => 10,
            _ => panic!("Bad card value!"),
        }
    }

    pub fn order(&self) -> u8 {
        match self.value {
            'J' => 11,
            'Q' => 12,
            'K' => 13,
            _ => self.score_value(),
        }
    }

    pub fn to_net_name(&self) -> String {
        format!(
            "{}{}",
            self.value,
            match self.suit {
                Suit::Spades => "S",
                Suit::Hearts => "H",
                Suit::Diamonds => "D",
                Suit::Clubs => "C",
            }
        )
    }

    pub fn from_net_name(name: String) -> Card {
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

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = &self.value.to_string();

        let val = match self.value {
            'T' => "10",
            _ => str,
        };

        write!(
            f,
            "{}{}",
            val,
            match self.suit {
                Suit::Spades => "♤",
                Suit::Hearts => "♥",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♧",
            }
        )
    }
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Deck {
        let cards: Vec<Card> = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs]
            .iter()
            .cartesian_product("A23456789TJQK".chars())
            .map(|(&suit, value)| Card { value, suit })
            .collect();

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

    pub fn draw_magic(&self) -> &Card {
        self.cards.last().unwrap()
    }

    pub fn rejoin(&mut self, mut hand: Hand) {
        self.cards.append(&mut hand.cards);
    }
}

#[derive(Clone)]
pub struct Hand {
    cards: Vec<Card>,
    magic: Option<Card>,
}

impl Hand {
    pub fn from(cards: Vec<Card>, magic: Option<Card>) -> Hand {
        Hand { cards, magic }
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

    pub fn remove_card(&mut self, card: &Card) {
        self.cards.retain(|c| c != card);
    }

    pub fn combine(&mut self, other: &mut Hand) {
        while other.len() > 0 {
            self.push(other.remove(0))
        }
    }

    pub fn set_magic(&mut self, magic: Card) {
        self.magic = Some(magic);
    }

    // Scores the hand for the 'Show' round, includes magic card
    pub fn score(&self) -> u8 {
        let mut score: u8 = 0;

        // Nob
        let jack: Vec<&Card> = self
            .cards
            .iter()
            .filter(|card| card.suit == self.magic.clone().unwrap().suit)
            .filter(|card| card.value == 'J')
            .collect();

        if jack.len() == 1 {
            score += 1;

            std::thread::sleep(std::time::Duration::from_millis(250));

            println!(
                "Nob for {score}! ({}, {})",
                jack.first().unwrap(),
                self.magic.as_ref().unwrap()
            );
        }

        let mut full_hand = vec![self.magic.clone().unwrap()];
        full_hand.extend_from_slice(&self.cards);
        full_hand.sort_by(|a, b| a.order().cmp(&b.order()));

        for perm in (2..=full_hand.len())
            .collect::<Vec<usize>>()
            .iter()
            .map(|len| full_hand.iter().combinations(*len))
            .flatten()
        {
            // Pairs
            if perm.len() == 2 && (perm[0].value == perm[1].value) {
                score += 2;

                std::thread::sleep(std::time::Duration::from_millis(250));
                println!("Pair for {score}! ({}, {})", perm[0], perm[1]);
            }

            // Fifteens
            if perm.iter().map(|card| card.score_value()).sum::<u8>() == 15 {
                score += 2;

                std::thread::sleep(std::time::Duration::from_millis(250));
                println!("Fifteen for {score}! ({})", perm.iter().join(", "));
            }

            // Runs
            if perm.len() >= 3 {
                let mut run = true;
                let mut last = perm[0].order();

                for card in perm.iter().skip(1) {
                    if card.order() != last + 1 {
                        run = false;
                        break;
                    }

                    last = card.order();
                }

                if run {
                    score += perm.len() as u8;
                std::thread::sleep(std::time::Duration::from_millis(250));
                    println!("Run for {score}! ({})", perm.iter().join(", "));
                }
            }
        }

        score
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

    pub fn magic(&self) -> Option<&Card> {
        self.magic.as_ref()
    }
}

impl fmt::Display for Hand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();

        for card in &self.cards {
            s.push_str(format!("{},", card).as_str());
        }

        write!(f, "{}", s)
    }
}

// Takes a sorted sliced of cards and returns true if they are a run
pub fn is_run(cards: &[&Card]) -> bool {
    if cards.len() < 3 {
        return false;
    }

    let mut last = cards[0].order();

    for card in cards.iter().skip(1) {
        if card.order() != last + 1 {
            return false;
        }

        last = card.order();
    }

    true
}
