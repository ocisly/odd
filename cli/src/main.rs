use fastrand::Rng;
use git_version::git_version;
use itertools::Itertools;
use odd_engine::{Card, Game, GameOutcome, GameState, Outcome, Player, HOLE_CARDS_PER_PLAYER};
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();
    let players = opt
        .hole_cards
        .chunks_exact(HOLE_CARDS_PER_PLAYER)
        .map(|x| x.try_into().unwrap())
        .collect_vec();
    for (i, player) in players.iter().enumerate() {
        print!("player {:2} was dealt: ", i + 1);
        for card in player {
            print!("{} ", card);
        }
        println!();
    }

    let flop = opt.board.get(..3);
    let turn = opt.board.get(3..4);
    let river = opt.board.get(4..5);
    let board = ["flop", "turn", "river"]
        .into_iter()
        .zip([flop, turn, river].into_iter().flatten());

    for (name, cards) in board {
        print!("{}:", name);
        for card in cards {
            print!(" {}", *card);
        }
        println!();
    }

    let rng = Rng::with_seed(opt.seed);
    let rng = RngAdapter(rng);
    let n_players = players.len();
    let game = Game::new(players, opt.board, opt.opponents, opt.folded);
    let GameOutcome {
        state,
        cards_remaining,
    } = game.play(rng, opt.permutations)?;

    println!();

    println!("{} cards remain.", cards_remaining);

    println!();

    match state {
        GameState::GameOver(outcomes) => {
            for (i, outcome) in outcomes.into_iter().enumerate() {
                print!("player {:2} has {} ", i + 1, outcome.hand);
                match outcome.outcome {
                    Outcome::Win => print!("(winner)"),
                    Outcome::Tie => print!("(tie)"),
                    Outcome::Loss => print!("(lost)"),
                }
                println!();
            }
        }
        GameState::Undecided(all_odds) => {
            for odds in all_odds.merge_unknown_players(n_players).into_iter() {
                match odds.who {
                    Player::Single(id) => print!("   player {:2}: ", id),
                    Player::Multiple(count) => print!("{:2} opponents: ", count),
                }
                println!(
                    "win {:5.2}%, tie {:5.2}%, loss {:5.2}%",
                    odds.win_percent(),
                    odds.tie_percent(),
                    odds.loss_percent(),
                );
                if !opt.distribution {
                    continue;
                }
                for (hand_type, percent) in odds.distribution() {
                    println!("{:20}: {:5.2}%", *hand_type, percent);
                }
                println!();
            }
        }
    }
    Ok(())
}

struct RngAdapter(Rng);

impl odd_engine::Rng<usize> for RngAdapter {
    fn generate(&mut self, range: impl std::ops::RangeBounds<usize>) -> usize {
        self.0.usize(range)
    }
}
const VERSION: &str = git_version!();

#[derive(StructOpt)]
#[structopt(name = "odd", version = VERSION)]
/// Texas hold'em poker odds simulator
///
/// When all the players' hole cards and all five community cards are known, odd detects what hand
/// each player holds and determines the winners. Otherwise, odd estimates the odds of winning for
/// each player by generating a configurable number of random deck shuffles to simulate a range of
/// possible scenarios.
struct Opt {
    /// Pairs of hole cards for each (known) player; e.g. As Kd 5h Tc
    #[structopt(required = true, number_of_values = 2, multiple = true)]
    hole_cards: Vec<Card>,

    /// Community cards comprising the flop, turn, and river; e.g. 2s 3h 4c 5d 6s
    #[structopt(short, long, min_values = 3, max_values = 5)]
    board: Vec<Card>,

    /// Number of additional players with unknown hole cards
    #[structopt(short, long, default_value = "0")]
    opponents: usize,

    /// Number of additional players with unknown hole cards who have folded
    #[structopt(short, long, default_value = "0")]
    folded: usize,

    /// RNG seed used for generating permutations of the deck
    #[structopt(short, long, default_value = "1")]
    seed: u64,

    /// Number of deck permutations to generate
    #[structopt(short, long, default_value = "1000000")]
    permutations: usize,

    /// Whether to include hand distribution in the output
    #[structopt(short, long)]
    distribution: bool,
}
