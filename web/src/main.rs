use fastrand::Rng;
use odd_engine::{Card, Game, GameOutcome, GameState, HOLE_CARDS_PER_PLAYER};
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

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/run").post(run);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn run(mut req: Request<()>) -> tide::Result<Body> {
    let Input {
        players,
        board,
        iterations,
        opponents,
    } = req.body_json().await?;
    let rng = RngAdapter(Rng::with_seed(1));
    let game = Game::new(players, board, opponents.unwrap_or(0));
    let GameOutcome {
        state,
        cards_remaining,
    } = game.play(rng, iterations.unwrap_or(100_000))?;
    let output = match state {
        GameState::Undecided(odds) => {
            json!({
                "cards_remaining": cards_remaining,
                "odds": odds.into_iter().map(|o| {
                    let distribution = o.distribution()
                        .map(|(hand_type, value)| (hand_type.to_string(), format!("{:.2}%", value)))
                        .collect::<HashMap<_, _>>();
                    json!({
                        "win": format!("{:.2}%", o.win_percent()),
                        "loss": format!("{:.2}%",o.loss_percent()),
                        "tie": format!("{:.2}%",o.tie_percent()),
                        "distribution": distribution
                    }
                )}).collect::<Vec<_>>()
            })
        }
        GameState::GameOver(outcomes) => {
            json!({
                "cards_remaining": cards_remaining,
                "outcomes": outcomes.into_iter().map(|outcome| json!({
                    "outcome": format!("{:?}", outcome.outcome),
                    "hand_type": format!("{}", outcome.hand.hand_type),
                    "cards": outcome.hand.cards.map(|card| card.to_string()),
                })).collect::<Vec<_>>()
            })
        }
    };
    Body::from_json(&output)
}

struct RngAdapter(Rng);

impl odd_engine::Rng<usize> for RngAdapter {
    fn generate(&mut self, range: impl std::ops::RangeBounds<usize>) -> usize {
        self.0.usize(range)
    }
}
