use fastrand::Rng;
use odd_engine::{odds, outcomes, Card, Deck, Hand, HOLE_CARDS_PER_PLAYER};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use tide::prelude::*;
use tide::{Body, Request};

#[serde_as]
#[derive(Debug, Deserialize)]
struct Input {
    #[serde_as(as = "Vec<[DisplayFromStr; HOLE_CARDS_PER_PLAYER]>")]
    players: Vec<[Card; 2]>,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    board: Vec<Card>,
    iterations: Option<usize>,
    opponents: Option<usize>,
}

#[serde_as]
#[derive(Serialize)]
struct Output {
    #[serde_as(as = "Vec<DisplayFromStr>")]
    hands: Vec<Hand>,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/outcomes").post(run_outcomes);
    app.at("/odds").post(run_odds);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn run_outcomes(mut req: Request<()>) -> tide::Result<Body> {
    let Input { players, board, .. } = req.body_json().await?;
    let players = players.iter().map(|x| &x[..]).collect::<Vec<&[Card]>>();
    let outcomes = outcomes(&players, &board)
        .map(|o| o.hand)
        .collect::<Vec<_>>();
    let output = Output { hands: outcomes };
    Body::from_json(&output)
}

async fn run_odds(mut req: Request<()>) -> tide::Result<Body> {
    let Input {
        players,
        board,
        iterations,
        opponents,
    } = req.body_json().await?;
    let players = players.iter().map(|x| &x[..]).collect::<Vec<&[Card]>>();
    let mut deck = Deck::default();
    for player in &players {
        for card in *player {
            deck.remove(card).ok().expect("duplicate card");
        }
    }
    for card in &board {
        deck.remove(card).ok().expect("duplicate card");
    }
    let rng = RngAdapter(Rng::with_seed(1));
    let odds = odds(
        opponents.unwrap_or(0),
        &players,
        &board,
        deck,
        iterations.unwrap_or(1000),
        rng,
    )
    .into_iter();
    let output = json!({
        "odds": odds.map(|o| json!({
            "win": format!("{:.2}%", o.win_percent()),
            "loss": format!("{:.2}%",o.loss_percent()),
            "tie": format!("{:.2}%",o.tie_percent()),
            "distribution": o.distribution().map(|(hand_type, value)| (hand_type.to_string(), format!("{:.2}%", value))).collect::<HashMap<_, _>>()
        })).collect::<Vec<_>>()
    });
    Body::from_json(&output)
}

struct RngAdapter(Rng);

impl odd_engine::Rng<usize> for RngAdapter {
    fn generate(&mut self, range: impl std::ops::RangeBounds<usize>) -> usize {
        self.0.usize(range)
    }
}
