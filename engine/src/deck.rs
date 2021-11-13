use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::card::{Card, Rank, Suit};

pub struct Deck(HashSet<Card>);

#[derive(Debug)]
pub enum DeckError {
    DuplicateCard(Card),
}
impl Display for DeckError {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), std::fmt::Error> {
        match self {
            DeckError::DuplicateCard(card) => write!(fmt, "duplicate card: {}", card),
        }
    }
}
impl Error for DeckError {}

impl Deck {
    pub fn remove(&mut self, card: &Card) -> Result<(), DeckError> {
        if self.0.remove(card) {
            Ok(())
        } else {
            Err(DeckError::DuplicateCard(*card))
        }
    }

    pub fn consume(self) -> impl Iterator<Item = Card> {
        self.0.into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self(
            Suit::ALL
                .iter()
                .copied()
                .flat_map(|suit| {
                    Rank::ALL
                        .iter()
                        .copied()
                        .map(move |rank| Card { rank, suit })
                })
                .collect(),
        )
    }
}
