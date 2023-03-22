use rand::{seq::SliceRandom, thread_rng};

#[derive(Copy, Clone)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

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
        }
    }

    pub fn rejoin(&mut self, hand: &mut Hand) {
        self.cards.append(&mut hand.cards);
    }
}

pub struct Hand {
    pub cards: Vec<Card>,
}

impl Hand {
    pub fn score(&self) -> u8 {
        todo!()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn pretty_print(&self) {
        let mut lines = vec![
            "┌─────┐ ".repeat(self.len()),
            "│xz   │ ".repeat(self.len()),
            "│     │ ".repeat(self.len()),
            "│    y│ ".repeat(self.len()),
            "└─────┘ ".repeat(self.len()),
        ];

        for card in &self.cards {
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
                Suit::Spades => "♠",
                Suit::Hearts => "♥",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
            };

            let new = fourth_line.replacen("y", suit, 1);
            lines.remove(3);
            lines.insert(3, new);
        }

        for line in lines {
            println!("{}", line);
        }
    }

    pub fn simple_print(&self) {
        let mut s = String::new();

        for card in &self.cards {
            s.push_str(&card.to_short_name());
        }

        println!("{}", s);
    }
}
