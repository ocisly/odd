use calc::{odds, outcomes, Outcome};
use card::Rank::*;
use card::Suit::*;
use card::{Card, Rank, Suit};
use deck::Deck;
use fastrand::Rng;
use git_version::git_version;
use hand::HandType::*;
use hand::{hands, Hand, HandType};
use itertools::Itertools;
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
        let outcomes = outcomes(hands);
        for (i, outcome) in outcomes.enumerate() {
            print!("player {} has {} ", i + 1, outcome.hand);
            match outcome.outcome {
                Outcome::Win => print!("(winner)"),
                Outcome::Tie => print!("(tie)"),
                Outcome::Loss => print!("(lost)"),
            }
            println!();
        }
    } else {
        let rng = Rng::with_seed(opt.seed);
        for (i, odds) in odds(
            opt.opponents,
            players,
            opt.board,
            deck,
            opt.permutations,
            rng,
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
            if !opt.distribution {
                continue;
            }
            for (hand_type, percent) in odds.distribution() {
                println!("{:20}: {:5.2}%", hand_type, percent);
            }
            println!();
        }
    }
}

impl floyd::Rng<usize> for Rng {
    fn generate(&mut self, range: impl std::ops::RangeBounds<usize>) -> usize {
        self.usize(range)
    }
}

impl Display for HandType {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
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
        .fmt(fmt)
    }
}

impl Display for Hand {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}, ", self.hand_type)?;
        let high = self.cards[0].rank;
        match self.hand_type {
            StraightFlush => write!(fmt, "{} high", VerboseRank(high)),
            FourOfAKind => write!(fmt, "{}s", VerboseRank(high)),
            FullHouse => write!(
                fmt,
                "{} full of {}",
                PluralRank(high),
                PluralRank(self.cards[3].rank)
            ),
            Flush => write!(fmt, "{} high", VerboseRank(high)),
            Straight => write!(fmt, "{} high", VerboseRank(high)),
            ThreeOfAKind => write!(fmt, "{}", PluralRank(high)),
            TwoPair => write!(
                fmt,
                "{} and {}",
                PluralRank(high),
                PluralRank(self.cards[2].rank)
            ),
            Pair => write!(fmt, "{}", PluralRank(high)),
            HighCard => write!(fmt, "{}", VerboseRank(high)),
        }?;
        write!(fmt, ":")?;
        for card in self.cards {
            write!(fmt, " {}", card)?;
        }
        Ok(())
    }
}

struct VerboseRank(Rank);
impl Display for VerboseRank {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self.0 {
            Deuce => "Deuce",
            Trey => "Trey",
            Four => "Four",
            Five => "Five",
            Six => "Six",
            Seven => "Seven",
            Eight => "Eight",
            Nine => "Nine",
            Ten => "Ten",
            Jack => "Jack",
            Queen => "Queen",
            King => "King",
            Ace => "Ace",
        }
        .fmt(fmt)
    }
}

struct PluralRank(Rank);
impl Display for PluralRank {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self.0 {
            Deuce => "Deuces",
            Trey => "Treys",
            Four => "Fours",
            Five => "Fives",
            Six => "Sixes",
            Seven => "Sevens",
            Eight => "Eights",
            Nine => "Nines",
            Ten => "Tens",
            Jack => "Jacks",
            Queen => "Queens",
            King => "Kings",
            Ace => "Aces",
        }
        .fmt(fmt)
    }
}

const VERSION: &str = git_version!();

#[derive(StructOpt, Debug)]
#[structopt(name = "odd", version = VERSION)]
/// Texas hold'em poker odds simulator
///
/// When all the players' hole cards and all five community cards are known, odd detects what hand
/// each player holds and determines the winners. Otherwise, odd estimates the odds of winning for
/// each player by generating a configurable number of random deck shuffles to simulate a range of
/// possible scenarios.
struct Opt {
    /// Pairs of hole cards for each (known) player; e.g. As Kd 5h Tc
    #[structopt(required = true, number_of_values = 2, multiple = true)]
    hole_cards: Vec<Card>,

    /// Community cards comprising the flop, turn, and river; e.g. 2s 3h 4c 5d 6s
    #[structopt(short, long, min_values = 3, max_values = 5)]
    board: Vec<Card>,

    /// Number of additional players with unknown hole cards
    #[structopt(short, long, default_value = "0")]
    opponents: usize,

    /// RNG seed used for generating permutations of the deck
    #[structopt(short, long, default_value = "1")]
    seed: u64,

    /// Number of deck permutations to generate
    #[structopt(short, long, default_value = "1000000")]
    permutations: usize,

    /// Whether to include hand distribution in the output
    #[structopt(short, long)]
    distribution: bool,
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
        .fmt(fmt)
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
        match self {
            Hearts => "♥️",
            Clubs => "♣️",
            Spades => "♠️",
            Diamonds => "♦️",
        }
        .fmt(fmt)
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
