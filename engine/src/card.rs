use itertools::Itertools;
use Rank::*;
use Suit::*;

pub fn combine_cards(sources: &[&Cards]) -> Vec<Card> {
    sources.iter().copied().flatten().copied().collect_vec()
}

pub fn combine_players<'a>(players: &[&'a Players]) -> Vec<&'a Cards> {
    players.iter().copied().flatten().copied().collect_vec()
}

pub type Players<'a> = [&'a Cards];
pub type Cards = [Card];

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Rank {
    Deuce = 2,
    Trey,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Deuce, Trey, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
    ];

    #[cfg(test)]
    pub const ALL_WITH_BOTH_ACES: [Rank; 14] = [
        Ace, Deuce, Trey, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
    ];
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Suit {
    Hearts,
    Clubs,
    Spades,
    Diamonds,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Hearts, Clubs, Spades, Diamonds];
}