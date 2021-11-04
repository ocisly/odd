use crate::card::Rank::*;
use crate::card::{Card, Rank};
use itertools::Itertools;
use std::cmp::{Ordering, Reverse};
use std::collections::HashMap;
use HandType::*;

pub fn hands(players: Vec<Vec<Card>>, board: Vec<Card>) -> Vec<Hand> {
    players
        .iter()
        .map(|hole| hand(combine_cards(&[hole, &board])))
        .collect_vec()
}

pub fn combine_cards(sources: &[&[Card]]) -> Vec<Card> {
    sources.iter().copied().flatten().copied().collect_vec()
}

pub fn hand(mut cards: Vec<Card>) -> Hand {
    cards.sort_by_key(|x| Reverse(x.rank));
    let flush = find_flush(&cards);
    let ranks_by_count = invert(cards.iter().counts_by(|c| c.rank));
    let three = find_k(3, &cards, &ranks_by_count);
    let pairs = find_k(2, &cards, &ranks_by_count);

    find_straight_flush(&flush, &cards)
        .or_else(|| find_k(4, &cards, &ranks_by_count))
        .or_else(|| find_full_house(&three, &pairs))
        .or(flush)
        .or_else(|| find_straight(&cards))
        .or(three)
        .or(pairs)
        .or_else(|| find_high_card(&cards))
        .unwrap()
}

fn invert(counts: HashMap<Rank, usize>) -> HashMap<usize, Vec<Rank>> {
    counts
        .into_iter()
        .fold(HashMap::new(), |mut acc, (rank, count)| {
            acc.entry(count).or_insert_with(Vec::new).push(rank);
            acc
        })
}

fn find_straight_flush(flush: &Option<Hand>, all_cards: &[Card]) -> Option<Hand> {
    match flush {
        Some(Hand {
            hand_type: Flush,
            cards: flush_cards,
        }) => {
            let cards = all_cards
                .iter()
                .filter(|card| card.suit == flush_cards[0].suit)
                .copied()
                .collect_vec();
            let straight = find_straight(&cards)?;
            Some(Hand {
                hand_type: StraightFlush,
                ..straight
            })
        }
        _ => None,
    }
}

fn find_k(k: usize, cards: &[Card], ranks_by_count: &HashMap<usize, Vec<Rank>>) -> Option<Hand> {
    let ranks = ranks_by_count.get(&k)?;
    let (mut main_cards, kickers): (Vec<_>, Vec<_>) =
        cards.iter().partition(|x| ranks.contains(&x.rank));
    let hand_type = match (k, main_cards.len()) {
        (4, _) => FourOfAKind,
        (3, 3) => ThreeOfAKind,
        (3, 6) => FullHouse,
        (2, 4) => TwoPair,
        (2, 6) => TwoPair,
        (2, 2) => Pair,
        _ => None?,
    };
    main_cards.truncate(Hand::HAND_SIZE);
    main_cards.extend_from_slice(kickers.get(..Hand::HAND_SIZE - main_cards.len())?);

    Some(Hand {
        hand_type,
        cards: main_cards.try_into().ok()?,
    })
}

fn find_full_house(three: &Option<Hand>, pairs: &Option<Hand>) -> Option<Hand> {
    match (three, pairs) {
        (
            Some(Hand {
                hand_type: ThreeOfAKind,
                cards: three_cards,
            }),
            Some(Hand {
                hand_type: TwoPair | Pair,
                cards: pairs_cards,
            }),
        ) => Some(Hand {
            hand_type: FullHouse,
            cards: {
                let mut cards = *three_cards;
                cards[3..].copy_from_slice(&pairs_cards[..2]);
                cards
            },
        }),
        _ => None,
    }
}

fn find_high_card(cards: &[Card]) -> Option<Hand> {
    Some(Hand {
        hand_type: HighCard,
        cards: cards
            .iter()
            .take(Hand::HAND_SIZE)
            .copied()
            .collect::<Vec<_>>()
            .try_into()
            .ok()?,
    })
}

fn find_flush(cards: &[Card]) -> Option<Hand> {
    let mut flush = cards
        .iter()
        .copied()
        .into_group_map_by(|c| c.suit)
        .into_iter()
        .find(|(_, v)| v.len() >= 5)?
        .1;
    flush.truncate(Hand::HAND_SIZE);
    Some(Hand {
        hand_type: Flush,
        cards: flush.try_into().ok()?,
    })
}

fn find_straight(cards: &[Card]) -> Option<Hand> {
    let mut straight = cards
        .iter()
        .copied()
        .dedup_by(|card1, card2| card1.rank == card2.rank)
        .enumerate()
        .group_by(|(i, card)| card.rank as i16 - (cards.len() - *i) as i16)
        .into_iter()
        .map(|(_, group)| group.map(|(_i, card)| card).collect_vec())
        .find(|v| v.len() >= 4)?;
    if let ace @ Card { rank: Ace, .. } = cards.first()? {
        if let Card { rank: Deuce, .. } = straight.last()? {
            straight.push(*ace);
        }
    }
    straight.reverse();
    straight.truncate(Hand::HAND_SIZE);
    Some(Hand {
        hand_type: Straight,
        cards: straight.try_into().ok()?,
    })
}

#[cfg(test)]
pub fn gen_flushes() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::Suit;
    use std::iter::repeat;
    shuffled(&Rank::ALL)
        .combinations_with_replacement(5)
        .filter(|ranks| {
            let uniques = ranks.iter().unique().count();
            uniques >= 3
        })
        .flat_map(|ranks| {
            shuffled(&Suit::ALL).map(move |suit| {
                repeat(suit)
                    .zip(&ranks)
                    .map(|(suit, rank)| Card { suit, rank: *rank })
                    .collect()
            })
        })
}

#[cfg(test)]
pub fn gen_straights() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::Suit;
    shuffled(&Suit::ALL)
        .combinations_with_replacement(5)
        .filter(move |suits| suits.iter().unique().count() >= 2)
        .flat_map(move |suits| {
            Rank::ALL_WITH_BOTH_ACES
                .windows(5)
                .map(|window| window.to_owned())
                .map(move |ranks| {
                    shuffled(&ranks)
                        .zip(&suits)
                        .map(|(rank, suit)| Card { suit: *suit, rank })
                        .collect()
                })
        })
}

#[cfg(test)]
pub fn gen_straight_flushes() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::Suit;
    shuffled(&Suit::ALL).flat_map(move |suit| {
        Rank::ALL_WITH_BOTH_ACES
            .windows(5)
            .map(|window| window.to_owned())
            .map(move |ranks| {
                shuffled(&ranks)
                    .zip(std::iter::repeat(suit))
                    .map(|(rank, suit)| Card { suit, rank })
                    .collect()
            })
    })
}

#[cfg(test)]
fn shuffled<T: Copy>(things: &[T]) -> impl Iterator<Item = T> + '_ {
    let mut things = things.to_owned();
    let rng = fastrand::Rng::with_seed(1);
    let n = things.len();
    // for i from 0 to n−2 do
    (0..n).map(move |i| {
        // j ← random integer such that i ≤ j < n
        let j = rng.usize(i..n);
        // exchange a[i] and a[j]
        things.swap(i, j);
        things[i]
    })
}

#[derive(Debug, Clone, Eq)]
pub struct Hand {
    pub hand_type: HandType,
    pub cards: [Card; Hand::HAND_SIZE],
}

impl Hand {
    const HAND_SIZE: usize = 5;
}

impl PartialEq for Hand {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.hand_type, other.hand_type) {
            (a, b) if a > b => Ordering::Greater,
            (a, b) if a == b => self
                .cards
                .iter()
                .zip(&other.cards)
                .map(|(card1, card2)| card1.rank.cmp(&card2.rank))
                .find(|c| *c != Ordering::Equal)
                .unwrap_or(Ordering::Equal),
            _ => Ordering::Less,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, PartialOrd, Eq)]
pub enum HandType {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::Suit;
    use crate::card::Suit::*;
    use std::iter::repeat;

    fn parse_card(raw: &str) -> Card {
        raw.parse().unwrap()
    }

    fn parse_cards(raw: &str) -> Vec<Card> {
        raw.split(' ').map(parse_card).collect()
    }

    #[test]
    fn test_high_card() {
        let cards = parse_cards("2c 3c 4c 5c Kh");
        let result = hand(cards);
        assert_eq!(result.hand_type, HighCard);
        assert_eq!(result.cards[0], parse_card("Kh"));
    }

    #[test]
    fn test_pair() {
        for rank in Rank::ALL.iter().copied() {
            for (suit1, suit2) in Suit::ALL.iter().copied().tuple_combinations() {
                let card1 = Card { rank, suit: suit1 };
                let card2 = Card { rank, suit: suit2 };
                let kicker_ranks = Rank::ALL.iter().copied().filter(|x| *x != rank).take(3);
                let kickers = kicker_ranks
                    .zip(repeat(Hearts))
                    .map(|(rank, suit)| Card { rank, suit });
                let result = hand(kickers.chain(vec![card1, card2]).collect());
                assert_eq!(result.hand_type, Pair);
            }
        }
    }

    #[test]
    fn test_two_pair() {
        for (rank1, rank2, rank3) in Rank::ALL.iter().copied().tuple_combinations() {
            for (suit1, suit2, suit3) in Suit::ALL.iter().copied().tuple_combinations() {
                let card1 = Card {
                    rank: rank1,
                    suit: suit1,
                };
                let card2 = Card {
                    rank: rank1,
                    suit: suit2,
                };
                let card3 = Card {
                    rank: rank2,
                    suit: suit1,
                };
                let card4 = Card {
                    rank: rank2,
                    suit: suit2,
                };
                let card5 = Card {
                    rank: rank3,
                    suit: suit3,
                };
                let result = hand(vec![card1, card2, card3, card4, card5]);
                assert_eq!(result.hand_type, TwoPair);
            }
        }
    }

    #[test]
    fn test_two_pair_tricky() {
        let cards = parse_cards("Qh Qd Jh Jd 6d 6d");
        let result = hand(cards);
        assert_eq!(result.hand_type, TwoPair);
        assert_eq!(result.cards.to_vec(), parse_cards("Qh Qd Jh Jd 6d"));
    }

    #[test]
    fn test_three() {
        for rank in Rank::ALL.iter().copied() {
            for suits in Suit::ALL.iter().copied().combinations(3) {
                let kicker_ranks = Rank::ALL.iter().copied().filter(|x| *x != rank).take(2);
                let cards = suits
                    .into_iter()
                    .zip(repeat(rank))
                    .map(|(suit, rank)| Card { suit, rank });

                let kickers = kicker_ranks.map(|kicker_rank| Card {
                    suit: Hearts,
                    rank: kicker_rank,
                });

                let result = hand(cards.chain(kickers).collect());
                assert_eq!(result.hand_type, ThreeOfAKind);
            }
        }
    }

    #[test]
    fn test_straight() {
        for cards in gen_straights() {
            let result = hand(cards);
            assert_eq!(result.hand_type, Straight, "{:#?}", result);
        }
    }

    #[test]
    fn test_straight_with_pair() {
        let cards = parse_cards("As 2s 2d 3d 4s 5h 5c");
        let result = hand(cards);
        assert_eq!(result.hand_type, Straight);
        assert_eq!(result.cards.to_vec(), parse_cards("As 2s 3d 4s 5h"));
    }

    #[test]
    fn test_straight_flush_with_pair() {
        let cards = parse_cards("As 2d 2s 3s 4s 5s 5c");
        let result = hand(cards);
        assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
        assert_eq!(result.cards.to_vec(), parse_cards("As 2s 3s 4s 5s"));
    }

    #[test]
    fn test_flush() {
        for cards in gen_flushes() {
            let result = hand(cards);
            if result.hand_type == StraightFlush {
                continue;
            }
            assert_eq!(result.hand_type, Flush, "{:#?}", result);
        }
    }

    #[test]
    fn test_straight_flush() {
        for cards in gen_straight_flushes() {
            let result = hand(cards);
            assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
        }
    }

    #[test]
    fn test_straight_flush_with_extra_cards() {
        let cards = parse_cards("8s 5s 2s 3s 4s 6s");
        let result = hand(cards);
        assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
    }

    #[test]
    fn test_full_house() {
        for (rank1, rank2) in Rank::ALL.iter().copied().tuple_combinations() {
            for suits1 in Suit::ALL.iter().copied().combinations(3) {
                let cards1: Vec<_> = suits1.into_iter().zip(repeat(rank1)).collect();

                for suits2 in Suit::ALL.iter().copied().combinations(2) {
                    let cards2 = suits2.into_iter().zip(repeat(rank2));

                    let result = hand(
                        cards1
                            .iter()
                            .copied()
                            .chain(cards2)
                            .map(|(suit, rank)| Card { suit, rank })
                            .collect(),
                    );
                    assert_eq!(result.hand_type, FullHouse);
                }
            }
        }
    }

    #[test]
    fn test_tricky_full_house() {
        let cards = parse_cards("8h 8s 8c Ah As 9c Ac");
        let result = hand(cards);
        assert_eq!(result.hand_type, FullHouse);
        assert_eq!(result.cards.to_vec(), parse_cards("Ah As Ac 8h 8s"));
    }

    #[test]
    fn test_four() {
        for rank in Rank::ALL.iter().copied() {
            let cards = Suit::ALL
                .iter()
                .copied()
                .zip(repeat(rank))
                .map(|(suit, rank)| Card { suit, rank });
            let kicker_rank = Rank::ALL.iter().copied().find(|r| *r != rank).unwrap();
            let kicker = Card {
                suit: Hearts,
                rank: kicker_rank,
            };
            let result = hand(std::iter::once(kicker).chain(cards).collect());
            assert_eq!(result.hand_type, FourOfAKind);
        }
    }
}
