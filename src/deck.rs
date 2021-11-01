use std::collections::HashSet;

use crate::card::{Card, Rank, Suit};

pub struct Deck(HashSet<Card>);

impl Deck {
    pub fn remove(&mut self, card: &Card) {
        assert!(self.0.remove(card), "duplicate card: {}", card);
    }

    pub fn consume(self) -> impl Iterator<Item = Card> {
        self.0.into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
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
