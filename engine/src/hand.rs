use crate::card::Rank::*;
use crate::card::{Card, Cards, Players};
use crate::HOLE_CARDS_PER_PLAYER;
use itertools::Itertools;
use std::cmp::{Ordering, Reverse};
use std::iter::zip;
use HandType::*;

pub fn hands(players: &Players, board: &Cards) -> Vec<Hand> {
    players
        .iter()
        .map(|hole| hand([hole, board].concat()))
        .collect_vec()
}

pub fn hand(mut cards: Vec<Card>) -> Hand {
    cards.sort_unstable_by_key(|x| Reverse(x.rank));
    assert!(
        cards.iter().dedup_with_count().any(|(dupes, _)| dupes > 0),
        "bug: all cards must be unique"
    );
    assert!(
        cards.len() <= Hand::HAND_SIZE + HOLE_CARDS_PER_PLAYER,
        "bug: too many cards in hand"
    );
    let flush = find_flush(&cards);
    [
        find_straight_flush(&flush, &cards),
        flush,
        find_groups(&cards),
        find_straight(&cards),
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap_or_else(|| find_high_card(&cards))
}

fn find_straight_flush(flush: &Option<Hand>, all_cards: &Cards) -> Option<Hand> {
    match flush {
        Some(Hand {
            hand_type: Flush,
            cards: [Card { suit, .. }, ..],
        }) => {
            let cards = all_cards
                .iter()
                .filter(|card| card.suit == *suit)
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

fn find_groups(cards: &Cards) -> Option<Hand> {
    let (four, three, pairs) = groups(cards);
    match (&four[..], &three[..], &pairs[..]) {
        ([four], _, _) => Some(Hand {
            hand_type: FourOfAKind,
            cards: cards_for_hand(four.to_vec(), cards),
        }),
        ([], [three1, three2], _) => Some(Hand {
            hand_type: FullHouse,
            cards: cards_for_hand([*three1, *three2].concat(), cards),
        }),
        ([], [three], [two, ..]) => Some(Hand {
            hand_type: FullHouse,
            cards: cards_for_hand([&three[..], &two[..]].concat(), cards),
        }),
        ([], [three], []) => Some(Hand {
            hand_type: ThreeOfAKind,
            cards: cards_for_hand(three.to_vec(), cards),
        }),
        ([], [], [pair1, pair2, ..]) => Some(Hand {
            hand_type: TwoPair,
            cards: cards_for_hand([*pair1, *pair2].concat(), cards),
        }),
        ([], [], [pair]) => Some(Hand {
            hand_type: Pair,
            cards: cards_for_hand(pair.to_vec(), cards),
        }),
        ([], [], []) => None,
        _ => unreachable!("too many cards"),
    }
}

fn groups(cards: &Cards) -> (Vec<[Card; 4]>, Vec<[Card; 3]>, Vec<[Card; 2]>) {
    let grouped_cards = cards.iter().group_by(|c| c.rank);
    let (mut four, mut three, mut pairs) = (vec![], vec![], vec![]);
    for (_rank, cards) in grouped_cards.into_iter() {
        let cards = cards.copied().collect_vec();
        match cards.len() {
            4 => four.push(cards.try_into().unwrap()),
            3 => three.push(cards.try_into().unwrap()),
            2 => pairs.push(cards.try_into().unwrap()),
            _ => {}
        }
    }
    (four, three, pairs)
}

fn cards_for_hand(mut main_cards: Vec<Card>, all_cards: &[Card]) -> [Card; Hand::HAND_SIZE] {
    for card in all_cards {
        if !main_cards.contains(card) {
            main_cards.push(*card);
        }
    }
    main_cards[..Hand::HAND_SIZE].try_into().unwrap()
}

fn find_high_card(cards: &Cards) -> Hand {
    Hand {
        hand_type: HighCard,
        cards: cards[..Hand::HAND_SIZE].try_into().unwrap(),
    }
}

fn find_flush(cards: &Cards) -> Option<Hand> {
    let flush = cards
        .iter()
        .sorted_unstable_by_key(|c| c.suit as u8)
        .group_by(|c| c.suit)
        .into_iter()
        .map(|(_, group)| group.copied().collect_vec())
        .find(|v| v.len() >= Hand::HAND_SIZE)?;
    Some(Hand {
        hand_type: Flush,
        cards: flush[..Hand::HAND_SIZE].try_into().ok()?,
    })
}

fn find_straight(cards: &Cards) -> Option<Hand> {
    let mut straight = cards
        .iter()
        .dedup_by(|card1, card2| card1.rank == card2.rank)
        .enumerate()
        .group_by(|(i, card)| card.rank as i16 - (cards.len() - *i) as i16)
        .into_iter()
        .map(|(_, group)| group.map(|(_i, card)| *card).collect_vec())
        .find(|v| v.len() >= 4)?;
    if let (ace @ Card { rank: Ace, .. }, Card { rank: Deuce, .. }) =
        (cards.first()?, straight.last()?)
    {
        straight.push(*ace);
    }
    straight.truncate(Hand::HAND_SIZE);
    Some(Hand {
        hand_type: Straight,
        cards: straight.try_into().ok()?,
    })
}

#[cfg(test)]
pub fn gen_flushes() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::{Rank, Suit};
    use std::iter::repeat;
    shuffled(&Rank::ALL)
        .combinations(5)
        .filter(|ranks| {
            let deuce = ranks.contains(&Deuce);
            !ranks
                .iter()
                .sorted()
                .tuple_windows()
                .all(|(x, y)| (deuce && y == &Ace) || (*y as u8) - (*x as u8) == 1)
        })
        .flat_map(|ranks| {
            shuffled(&Suit::ALL).map(move |suit| {
                zip(repeat(suit), &ranks)
                    .map(|(suit, rank)| Card { suit, rank: *rank })
                    .collect()
            })
        })
}

#[cfg(test)]
pub fn gen_straights() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::{Rank, Suit};
    Suit::ALL
        .into_iter()
        .combinations_with_replacement(5)
        // filter out flushes
        .filter(move |suits| suits.iter().unique().count() > 1)
        // consider all the unique orders (permutations) the suits could be in
        .flat_map(move |suits| suits.into_iter().permutations(5).unique())
        // now consider all the possible straights for that permutation of suits
        .flat_map(move |suits| {
            Rank::ALL_WITH_BOTH_ACES
                // each straight is a sequence of 5 consecutive ranks
                .windows(5)
                .map(|window| window.to_owned())
                .map(move |ranks| {
                    zip(ranks, &suits)
                        .map(|(rank, suit)| Card { suit: *suit, rank })
                        .collect()
                })
        })
}

#[cfg(test)]
pub fn gen_straight_flushes() -> impl Iterator<Item = Vec<Card>> {
    use crate::card::{Rank, Suit};
    shuffled(&Suit::ALL).flat_map(move |suit| {
        Rank::ALL_WITH_BOTH_ACES
            .windows(5)
            .map(|window| window.to_owned())
            .map(move |ranks| {
                zip(shuffled(&ranks), std::iter::repeat(suit))
                    .map(|(rank, suit)| Card { suit, rank })
                    .collect()
            })
    })
}

#[cfg(test)]
fn shuffled<T: Copy>(things: &[T]) -> impl Iterator<Item = T> + '_ {
    let mut things = things.to_owned();
    let mut rng = fastrand::Rng::with_seed(1);
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
            (a, b) if a == b => zip(self.cards, &other.cards)
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
    use crate::{
        calc::HandTypeDistribution,
        card::{Rank, Suit},
        Deck,
    };
    use std::iter::repeat;

    fn parse_card(raw: &str) -> Card {
        raw.parse().unwrap()
    }

    fn parse_cards(raw: &str) -> Vec<Card> {
        raw.split(' ').map(parse_card).collect()
    }

    #[test]
    fn test_handles_all_valid_hands() {
        use rayon::prelude::*;

        let new_dist = HandTypeDistribution::default;

        let distribution = Deck::default()
            .consume()
            .combinations(7)
            .collect_vec()
            .into_par_iter()
            .map(|cards| hand(cards).hand_type)
            .fold(new_dist, HandTypeDistribution::update)
            .reduce(new_dist, HandTypeDistribution::merge);

        assert_eq!(41_584, distribution.frequency_of(&StraightFlush));
        assert_eq!(224_848, distribution.frequency_of(&FourOfAKind));
        assert_eq!(3_473_184, distribution.frequency_of(&FullHouse));
        assert_eq!(4_047_644, distribution.frequency_of(&Flush));
        assert_eq!(6_180_020, distribution.frequency_of(&Straight));
        assert_eq!(6_461_620, distribution.frequency_of(&ThreeOfAKind));
        assert_eq!(31_433_400, distribution.frequency_of(&TwoPair));
        assert_eq!(58_627_800, distribution.frequency_of(&Pair));
        assert_eq!(23_294_460, distribution.frequency_of(&HighCard));
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
        let mut n = 0;
        for rank in Rank::ALL.into_iter() {
            for (suit1, suit2) in Suit::ALL.into_iter().tuple_combinations() {
                let card1 = Card { rank, suit: suit1 };
                let card2 = Card { rank, suit: suit2 };
                for kicker_ranks in Rank::ALL.into_iter().filter(|x| *x != rank).combinations(3) {
                    for kicker_suits in Suit::ALL
                        .into_iter()
                        .combinations_with_replacement(3)
                        .flat_map(|s| s.into_iter().permutations(3).unique())
                    {
                        let kickers = zip(kicker_ranks.clone(), kicker_suits)
                            .map(|(rank, suit)| Card { rank, suit });
                        let result = hand(kickers.chain(vec![card1, card2]).collect());
                        assert_eq!(result.hand_type, Pair);
                        n += 1;
                    }
                }
            }
        }
        assert_eq!(1_098_240, n);
    }

    #[test]
    fn test_two_pair() {
        let mut n = 0;
        for (rank1, rank2) in Rank::ALL.into_iter().tuple_combinations() {
            for suits1 in Suit::ALL.into_iter().combinations(2) {
                for suits2 in Suit::ALL.into_iter().combinations(2) {
                    for kicker_rank in Rank::ALL
                        .into_iter()
                        .filter(|kicker_rank| *kicker_rank != rank1 && *kicker_rank != rank2)
                    {
                        for kicker_suit in Suit::ALL {
                            let mut cards = zip(
                                vec![rank1, rank1, rank2, rank2],
                                vec![suits1.clone(), suits2.clone()].into_iter().flatten(),
                            )
                            .map(|(rank, suit)| Card { rank, suit })
                            .collect_vec();
                            let kicker = Card {
                                rank: kicker_rank,
                                suit: kicker_suit,
                            };
                            cards.push(kicker);

                            let result = hand(cards);
                            assert_eq!(result.hand_type, TwoPair);

                            n += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(123_552, n);
    }

    #[test]
    fn test_two_pair_tricky() {
        let cards = parse_cards("Qh Qd Jh Jd 6d 6c");
        let result = hand(cards);
        assert_eq!(result.hand_type, TwoPair);
        assert_eq!(result.cards.to_vec(), parse_cards("Qh Qd Jh Jd 6d"));
    }

    #[test]
    fn test_three() {
        let mut n = 0;
        for rank in Rank::ALL.into_iter() {
            for suits in Suit::ALL.into_iter().combinations(3) {
                for kicker_ranks in Rank::ALL
                    .into_iter()
                    .filter(|kicker_rank| *kicker_rank != rank)
                    .combinations(2)
                {
                    for kicker_suits in Suit::ALL
                        .into_iter()
                        .combinations_with_replacement(2)
                        .flat_map(|combos| combos.into_iter().permutations(2))
                        .unique()
                    {
                        let cards = zip(suits.clone(), repeat(rank))
                            .map(|(suit, rank)| Card { suit, rank });
                        let kickers = zip(kicker_ranks.clone(), kicker_suits)
                            .map(|(rank, suit)| Card { rank, suit });

                        let result = hand(cards.chain(kickers.into_iter()).collect());
                        assert_eq!(result.hand_type, ThreeOfAKind);

                        n += 1;
                    }
                }
            }
        }
        assert_eq!(54_912, n);
    }

    #[test]
    fn test_straight() {
        let mut n = 0;
        for cards in gen_straights() {
            let result = hand(cards);
            n += 1;
            assert_eq!(result.hand_type, Straight, "{:#?}", result);
        }
        assert_eq!(10_200, n);
    }

    #[test]
    fn test_straight_with_pair() {
        let cards = parse_cards("As 2s 2d 3d 4s 5h 5c");
        let result = hand(cards);
        assert_eq!(result.hand_type, Straight);
        assert_eq!(result.cards.to_vec(), parse_cards("5h 4s 3d 2s As"));
    }

    #[test]
    fn test_elongated_straight() {
        let cards = parse_cards("As 2s 2d 3d 4s 5h 6c");
        let result = hand(cards);
        assert_eq!(result.hand_type, Straight);
        assert_eq!(result.cards.to_vec(), parse_cards("6c 5h 4s 3d 2s"));
    }

    #[test]
    fn test_straight_flush_with_pair() {
        let cards = parse_cards("As 2d 2s 3s 4s 5s 5c");
        let result = hand(cards);
        assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
        assert_eq!(result.cards.to_vec(), parse_cards("5s 4s 3s 2s As"));
    }

    #[test]
    fn test_flush() {
        let mut n = 0;
        for cards in gen_flushes() {
            let result = hand(cards);
            n += 1;
            assert_eq!(result.hand_type, Flush, "{:#?}", result);
        }
        assert_eq!(5108, n, "didn't generate all flushes");
    }

    #[test]
    fn test_straight_flush() {
        let mut n = 0;
        for cards in gen_straight_flushes() {
            let result = hand(cards);
            n += 1;
            assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
        }
        assert_eq!(40, n, "didn't generate all straight flushes");
    }

    #[test]
    fn test_straight_flush_with_extra_cards() {
        let cards = parse_cards("8s 5s 2s 3s 4s 6s");
        let result = hand(cards);
        assert_eq!(result.hand_type, StraightFlush, "{:#?}", result);
    }

    #[test]
    fn test_full_house() {
        let mut n = 0;
        for ranks in Rank::ALL.into_iter().permutations(2) {
            let (rank1, rank2) = ranks.into_iter().collect_tuple().unwrap();
            for suits1 in Suit::ALL.into_iter().combinations(3) {
                let cards1 = zip(suits1, repeat(rank1)).collect_vec();

                for suits2 in Suit::ALL.into_iter().combinations(2) {
                    let cards2 = zip(suits2, repeat(rank2));

                    let result = hand(
                        cards1
                            .iter()
                            .copied()
                            .chain(cards2)
                            .map(|(suit, rank)| Card { suit, rank })
                            .collect(),
                    );

                    n += 1;
                    assert_eq!(result.hand_type, FullHouse);
                }
            }
        }
        assert_eq!(3744, n, "didn't generate all full houses");
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
        let mut n = 0;
        for ranks in Rank::ALL.into_iter().permutations(2) {
            let (rank, kicker_rank) = ranks.into_iter().collect_tuple().unwrap();
            for kicker_suit in Suit::ALL {
                let cards = zip(Suit::ALL, repeat(rank)).map(|(suit, rank)| Card { suit, rank });
                let kicker = Card {
                    suit: kicker_suit,
                    rank: kicker_rank,
                };
                let result = hand(std::iter::once(kicker).chain(cards).collect());
                n += 1;
                assert_eq!(result.hand_type, FourOfAKind);
            }
        }
        assert_eq!(624, n, "didn't generate all quads");
    }
}
