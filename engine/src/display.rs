use crate::card::{Card, Rank, Suit};
use crate::hand::{Hand, HandType};
use std::fmt::{Display, Formatter};
use HandType::*;
use Rank::*;
use Suit::*;

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

impl Display for Card {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}{}", self.rank, self.suit)
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
