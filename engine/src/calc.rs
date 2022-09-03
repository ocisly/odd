use crate::card::{Cards, Players, HOLE_CARDS_PER_PLAYER};
use crate::deck::Deck;
use crate::floyd::permutations;
use crate::floyd::Rng;
use crate::hand::{hands, Hand, HandType};
use itertools::Itertools;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use Outcome::*;

pub const BOARD_LENGTH: usize = 5;

pub fn odds(
    opponents: usize,
    n_folded: usize,
    players: &Players,
    board: &Cards,
    deck: Deck,
    desired_samples: usize,
    rng: impl Rng<usize> + Send,
) -> Odds
where
{
    let unknown_hole_cards = HOLE_CARDS_PER_PLAYER * (opponents + n_folded);
    let unknown_board_cards = BOARD_LENGTH - board.len();
    let unknown_cards = unknown_hole_cards + unknown_board_cards;
    let new_odds = || Odds::new(opponents + players.len());
    permutations(unknown_cards, deck.consume().collect_vec(), rng)
        .take(desired_samples)
        .par_bridge()
        .map(|scenario| {
            let (extra_hole, extra_board) = scenario.split_at(unknown_hole_cards);
            let extra_players = extra_hole
                .chunks_exact(HOLE_CARDS_PER_PLAYER)
                .skip(n_folded)
                .map(|x| x.try_into().unwrap())
                .collect_vec();

            let all_players = [players, &extra_players].concat();
            let community_cards = [board, extra_board].concat();
            outcomes(&all_players, &community_cards)
        })
        .fold(new_odds, Odds::update)
        .reduce(new_odds, Odds::merge)
}

pub fn outcomes(players: &Players, board: &Cards) -> impl Iterator<Item = HandOutcome> {
    hand_outcomes(hands(players, board))
}

fn hand_outcomes(hands: Vec<Hand>) -> impl Iterator<Item = HandOutcome> {
    let max = hands.iter().max().unwrap().clone();
    let n_winners = hands.iter().filter(|x| **x == max).count();
    let win = if n_winners == 1 { Win } else { Tie };

    hands.into_iter().map(move |hand| HandOutcome {
        outcome: if hand == max { win } else { Loss },
        hand,
    })
}

pub struct HandOutcome {
    pub outcome: Outcome,
    pub hand: Hand,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Outcome {
    Win,
    Tie,
    Loss,
}

#[derive(Debug)]
pub struct Odds(Vec<HandOdds>);

impl Odds {
    fn new(num_players: usize) -> Self {
        Self((1..=num_players as u64).map(HandOdds::new).collect())
    }

    fn update(self, outcomes: impl Iterator<Item = HandOutcome>) -> Self {
        Self(
            outcomes
                .zip(self.0)
                .map(|(outcome, odds)| odds.update(outcome))
                .collect(),
        )
    }

    fn merge(self, other: Self) -> Self {
        Self(
            self.0
                .into_iter()
                .zip(other.0)
                .map(|(odds1, odds2)| odds1.merge(odds2))
                .collect(),
        )
    }

    pub fn merge_unknown_players(mut self, n: usize) -> Self {
        if self.0.len() - n <= 1 {
            return self;
        }
        let unknown = self.0.split_off(n).into_iter();
        let merged = unknown.reduce(HandOdds::merge).unwrap();
        self.0.push(merged);
        self
    }
}

impl IntoIterator for Odds {
    type Item = HandOdds;
    type IntoIter = <Vec<HandOdds> as IntoIterator>::IntoIter;

    fn into_iter(self) -> <Self as IntoIterator>::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug)]
pub enum Player {
    Single(u64),
    Multiple(usize),
}

#[derive(Clone, Debug)]
pub struct HandOdds {
    pub who: Player,
    wins: u64,
    ties: u64,
    losses: u64,
    distribution: HashMap<HandType, u64>,
}

impl HandOdds {
    pub fn new(id: u64) -> Self {
        Self {
            who: Player::Single(id),
            wins: 0,
            ties: 0,
            losses: 0,
            distribution: Default::default(),
        }
    }

    pub fn update(mut self, outcome: HandOutcome) -> Self {
        match outcome.outcome {
            Outcome::Win => self.wins += 1,
            Outcome::Tie => self.ties += 1,
            Outcome::Loss => self.losses += 1,
        }
        *self.distribution.entry(outcome.hand.hand_type).or_insert(0) += 1;
        self
    }

    pub fn merge(mut self, other: HandOdds) -> Self {
        self.who = match (self.who, other.who) {
            (Player::Single(id1), Player::Single(id2)) if id1 == id2 => Player::Single(id1),
            (Player::Single(_), Player::Single(_)) => Player::Multiple(2),
            (Player::Multiple(n), Player::Single(_)) => Player::Multiple(n + 1),
            (Player::Single(_), Player::Multiple(n)) => Player::Multiple(n + 1),
            (Player::Multiple(n), Player::Multiple(m)) => Player::Multiple(n + m),
        };
        self.wins += other.wins;
        self.ties += other.ties;
        self.losses += other.losses;
        for (hand_type, count) in other.distribution {
            *self.distribution.entry(hand_type).or_insert(0) += count;
        }
        self
    }

    pub fn win_percent(&self) -> f64 {
        100f64 * (self.wins as f64 / self.all() as f64)
    }

    pub fn tie_percent(&self) -> f64 {
        100f64 * (self.ties as f64 / self.all() as f64)
    }

    pub fn loss_percent(&self) -> f64 {
        100f64 * (self.losses as f64 / self.all() as f64)
    }

    pub fn all(&self) -> u64 {
        self.wins + self.ties + self.losses
    }

    pub fn distribution(&self) -> impl Iterator<Item = (&HandType, f64)> {
        self.distribution
            .iter()
            .sorted_by_key(|(_hand_type, count)| Reverse(**count))
            .map(|(hand_type, count)| (hand_type, 100f64 * (*count as f64 / self.all() as f64)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Card;
    use crate::hand::{gen_flushes, gen_straights, hand};

    fn parse_card(raw: &str) -> Card {
        raw.parse().unwrap()
    }

    fn parse_cards(raw: &str) -> Vec<Card> {
        raw.split(' ').map(parse_card).collect()
    }

    fn winners(hands: &[Hand]) -> Vec<Hand> {
        hand_outcomes(hands.to_vec())
            .filter(|o| o.outcome == Win)
            .map(|o| o.hand)
            .collect()
    }

    fn ties(hands: &[Hand]) -> Vec<Hand> {
        hand_outcomes(hands.to_vec())
            .filter(|o| o.outcome == Tie)
            .map(|o| o.hand)
            .collect()
    }

    #[test]
    fn test_single_hand_winner() {
        let cards = parse_cards("As 2d 4h 6c 8s");
        let hands = [hand(cards)];
        let winners = winners(&hands);
        assert_eq!(winners.len(), 1);
    }

    #[test]
    fn test_higher_card_wins() {
        let cards_winner = parse_cards("As 2d 4d 6d 8d");
        let cards_loser = parse_cards("Ks 2d 4d 6d 8d");
        let hands = [hand(cards_winner), hand(cards_loser)];
        let winners = winners(&hands);
        assert_eq!(winners.len(), 1);
        assert!(winners.contains(&hands[0]));
    }

    #[test]
    fn test_tie() {
        let cards1 = parse_cards("As Td 8d 6d 4d");
        let cards2 = parse_cards("Ac Td 8d 6d 4d");
        let hands = [hand(cards1), hand(cards2)];
        let ties = ties(&hands);
        assert_eq!(ties.len(), 2);
    }

    #[test]
    fn test_tie_high_kicker() {
        let cards1 = parse_cards("As Kd Th 8h 6h 4h");
        let cards2 = parse_cards("Ac Qd Th 8h 6h 4h");
        let hands = [hand(cards1), hand(cards2)];
        let winners = winners(&hands);
        assert_eq!(winners.len(), 1);
        assert!(winners.contains(&hands[0]));
    }

    #[test]
    fn test_tie_3way() {
        let cards1 = parse_cards("As 8d 6d 4d 2d");
        let cards2 = parse_cards("Ac 8d 6d 4d 2d");
        let cards3 = parse_cards("Ah 8d 6d 4d 2d");
        let hands = [hand(cards1), hand(cards2), hand(cards3)];
        let ties = ties(&hands);
        assert_eq!(ties.len(), 3);
    }

    #[test]
    fn test_better_hand_type_wins() {
        let cards1 = parse_cards("As Ah 2d 4d 6d");
        let cards2 = parse_cards("Ac Kc 2s 4s 6s");
        let hands = [hand(cards1), hand(cards2)];
        let winners = winners(&hands);
        assert_eq!(winners.len(), 1);
        assert!(winners.contains(&hands[0]));
    }

    #[test]
    fn flush_beats_straight() {
        for flush in gen_flushes().take(1000) {
            for straight in gen_straights().take(1000) {
                let hands = [hand(flush.clone()), hand(straight)];
                let winners = winners(&hands);
                assert!(winners.contains(&hands[0]));
            }
        }
    }

    #[test]
    fn test_better_flush_wins() {
        let cards1 = parse_cards("8s 6h As Ks Qs Js 5s");
        let cards2 = parse_cards("7s 6c As Ks Qs Js 5s");
        let hands = [hand(cards1), hand(cards2)];
        let winners = winners(&hands);
        assert!(winners.contains(&hands[0]));
        assert_eq!(winners.len(), 1);
    }
}
