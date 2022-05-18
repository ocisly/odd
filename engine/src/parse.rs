use crate::card::{Card, Rank, Suit};
use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use Rank::*;
use Suit::*;

impl FromStr for Card {
    type Err = CardParseError;

    fn from_str(string: &str) -> Result<Card, CardParseError> {
        match string.len() {
            0 => Err(CardParseError::EmptyCard),
            1 => Err(CardParseError::CardTooShort(string.into())),
            2 => Ok(Self {
                rank: string[0..1].parse()?,
                suit: string[1..2].parse()?,
            }),
            _ => Err(CardParseError::CardTooLong(string.into())),
        }
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

#[derive(Debug)]
pub enum CardParseError {
    InvalidSuit(String),
    InvalidRank(String),
    EmptyCard,
    CardTooShort(String),
    CardTooLong(String),
}

impl Error for CardParseError {}

impl Display for CardParseError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            CardParseError::InvalidSuit(s) => write!(fmt, "unknown suit: {}", s),
            CardParseError::InvalidRank(s) => write!(fmt, "unknown rank: {}", s),
            CardParseError::EmptyCard => write!(fmt, "empty card"),
            CardParseError::CardTooShort(s) => write!(fmt, "too short: {}", s),
            CardParseError::CardTooLong(s) => write!(fmt, "too long: {}", s),
        }
    }
}
