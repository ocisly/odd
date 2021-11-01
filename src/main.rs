use calc::{odds, outcomes, Outcome};
use card::Rank::*;
use card::Suit::*;
use card::{Card, Rank, Suit};
use deck::Deck;
use hand::HandType::*;
use hand::{hands, Hand, HandType};
use itertools::Itertools;
use oorandom::Rand64;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;
use structopt::StructOpt;

mod calc;
mod card;
mod deck;
mod floyd;
mod hand;

fn main() {
    let opt = Opt::from_args();
    let players = opt
        .hole_cards
        .chunks_exact(2)
        .map(|x| x.to_vec())
        .collect_vec();
    let mut deck = Deck::default();

    for (i, player) in players.iter().enumerate() {
        print!("player {} was dealt: ", i + 1);
        for card in player {
            deck.remove(card);
            print!("{} ", card);
        }
        println!();
    }

    println!();

    let flop = opt.board.get(..3);
    let turn = opt.board.get(3..4);
    let river = opt.board.get(4..5);
    let board = ["flop", "turn", "river"]
        .into_iter()
        .zip([flop, turn, river].into_iter().flatten());

    for (name, cards) in board {
        print!("{}:", name);
        for card in cards {
            print!(" {}", card);
            deck.remove(card);
        }
        println!();
    }

    println!();
    print!("{} cards remain.", deck.len());

    println!();
    println!();

    if opt.board.len() == 5 && opt.opponents == 0 {
        let hands = hands(players, opt.board);
        let outcomes = outcomes(&hands);
        for (i, (hand, outcome)) in hands.iter().zip(outcomes).enumerate() {
            print!("player {} has {}: ", i + 1, hand);
            for card in hand.cards {
                print!("{} ", card);
            }
            match outcome {
                Outcome::Win => print!("(winner)"),
                Outcome::Tie => print!("(tie)"),
                Outcome::Loss => print!("(lost)"),
            }
            println!();
        }
    } else {
        let mut rng = Rand64::new(opt.seed);
        for (i, odds) in odds(
            opt.opponents,
            players,
            opt.board,
            deck,
            opt.permutations,
            |bounds| rng.rand_range(bounds),
        )
        .into_iter()
        .enumerate()
        {
            println!(
                "player {}: win {:5.2}%, tie {:5.2}%, loss {:5.2}%",
                i + 1,
                odds.win_percent(),
                odds.tie_percent(),
                odds.loss_percent(),
            );
        }
    }
}

impl Display for HandType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                StraightFlush => "Straight Flush",
                FourOfAKind => "Four of a Kind",
                FullHouse => "Full House",
                Flush => "Flush",
                Straight => "Straight",
                ThreeOfAKind => "Three of a Kind",
                TwoPair => "Two Pair",
                Pair => "Pair",
                HighCard => "High Card",
            }
        )
    }
}

impl Display for Hand {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}, ", self.hand_type)?;
        let high = self.cards[0];
        match self.hand_type {
            StraightFlush => write!(fmt, "{} high", high.rank),
            FourOfAKind => write!(fmt, "{}s", high.rank),
            FullHouse => write!(fmt, "{}s full of {}s", high.rank, self.cards[3].rank),
            Flush => write!(fmt, "{} high", high.rank),
            Straight => write!(fmt, "{} high", high.rank),
            ThreeOfAKind => write!(fmt, "{}s", high.rank),
            TwoPair => write!(fmt, "{}s and {}s", high.rank, self.cards[2].rank),
            Pair => write!(fmt, "{}s", high.rank),
            HighCard => write!(fmt, "{}", high.rank),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "odd")]
struct Opt {
    #[structopt(required = true, number_of_values = 2, multiple = true)]
    hole_cards: Vec<Card>,
    #[structopt(short, long, min_values = 3, max_values = 5)]
    board: Vec<Card>,
    #[structopt(short, long, default_value = "0")]
    opponents: usize,
    #[structopt(short, long, default_value = "1")]
    seed: u128,
    #[structopt(short, long, default_value = "1000000")]
    permutations: usize,
}

impl FromStr for Card {
    type Err = CardParseError;

    fn from_str(string: &str) -> Result<Card, CardParseError> {
        let rank = string[0..1].parse()?;
        let suit = string[1..2].parse()?;
        Ok(Card { rank, suit })
    }
}

impl Display for Card {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}{}", self.rank, self.suit)
    }
}

impl FromStr for Rank {
    type Err = CardParseError;

    fn from_str(string: &str) -> Result<Rank, CardParseError> {
        match string {
            "2" => Ok(Deuce),
            "3" => Ok(Trey),
            "4" => Ok(Four),
            "5" => Ok(Five),
            "6" => Ok(Six),
            "7" => Ok(Seven),
            "8" => Ok(Eight),
            "9" => Ok(Nine),
            "T" => Ok(Ten),
            "J" => Ok(Jack),
            "Q" => Ok(Queen),
            "K" => Ok(King),
            "A" => Ok(Ace),
            _ => Err(CardParseError::InvalidRank(string.into())),
        }
    }
}

impl Display for Rank {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                Deuce => "2",
                Trey => "3",
                Four => "4",
                Five => "5",
                Six => "6",
                Seven => "7",
                Eight => "8",
                Nine => "9",
                Ten => "T",
                Jack => "J",
                Queen => "Q",
                King => "K",
                Ace => "A",
            }
        )
    }
}

impl FromStr for Suit {
    type Err = CardParseError;

    fn from_str(string: &str) -> Result<Suit, CardParseError> {
        match string {
            "h" => Ok(Hearts),
            "c" => Ok(Clubs),
            "s" => Ok(Spades),
            "d" => Ok(Diamonds),
            _ => Err(CardParseError::InvalidSuit(string.into())),
        }
    }
}

impl Display for Suit {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            match self {
                Hearts => "♥️",
                Clubs => "♣️",
                Spades => "♠️",
                Diamonds => "♦️",
            }
        )
    }
}

#[derive(Debug)]
pub enum CardParseError {
    InvalidSuit(String),
    InvalidRank(String),
}

impl Display for CardParseError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            CardParseError::InvalidSuit(s) => write!(fmt, "unknown suit: {}", s),
            CardParseError::InvalidRank(s) => write!(fmt, "unknown rank: {}", s),
        }
    }
}
