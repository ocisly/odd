use crate::calc::{odds, outcomes, HandOutcome, Odds};
use crate::card::{Card, HoleCards};
use crate::deck::{Deck, DeckError};
use crate::Rng;

const BOARD_LENGTH: usize = 5;

pub struct Game {
    players: Vec<HoleCards>,
    board: Vec<Card>,
    opponents: usize,
}

pub enum GameState {
    GameOver(Vec<HandOutcome>),
    Undecided(Odds),
}

pub struct GameOutcome {
    pub state: GameState,
    pub cards_remaining: usize,
}

impl Game {
    pub fn new(players: Vec<HoleCards>, board: Vec<Card>, opponents: usize) -> Self {
        Game {
            players,
            board,
            opponents,
        }
    }

    pub fn play(
        &self,
        rng: impl Rng<usize> + Send,
        permutations: usize,
    ) -> Result<GameOutcome, DeckError> {
        let mut deck = Deck::default();
        for player in &self.players {
            for card in player {
                deck.remove(card)?;
            }
        }
        for card in &self.board {
            deck.remove(card)?;
        }
        let cards_remaining = deck.len();
        Ok(if self.is_over() {
            GameOutcome {
                state: GameState::GameOver(outcomes(&self.players, &self.board).collect()),
                cards_remaining,
            }
        } else {
            GameOutcome {
                state: GameState::Undecided(odds(
                    self.opponents,
                    &self.players,
                    &self.board,
                    deck,
                    permutations,
                    rng,
                )),
                cards_remaining,
            }
        })
    }

    fn is_over(&self) -> bool {
        self.board.len() == BOARD_LENGTH && self.opponents == 0
    }
}
