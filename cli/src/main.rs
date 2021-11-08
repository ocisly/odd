use fastrand::Rng;
use git_version::git_version;
use itertools::Itertools;
use odd_engine::{odds, outcomes, Card, Deck, Outcome, BOARD_LENGTH, HOLE_CARDS_PER_PLAYER};
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    let players = opt
        .hole_cards
        .chunks_exact(HOLE_CARDS_PER_PLAYER)
        .collect_vec();
    let mut deck = Deck::default();

    for (i, player) in players.iter().enumerate() {
        print!("player {} was dealt: ", i + 1);
        for card in *player {
            deck.remove(card).ok().expect("duplicate card!");
            print!("{} ", *card);
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
            deck.remove(card).ok().expect("duplicate card!");
            print!(" {}", *card);
        }
        println!();
    }

    println!();
    print!("{} cards remain.", deck.len());

    println!();
    println!();

    if opt.board.len() == BOARD_LENGTH && opt.opponents == 0 {
        let outcomes = outcomes(&players, &opt.board);
        for (i, outcome) in outcomes.enumerate() {
            print!("player {} has {} ", i + 1, outcome.hand);
            match outcome.outcome {
                Outcome::Win => print!("(winner)"),
                Outcome::Tie => print!("(tie)"),
                Outcome::Loss => print!("(lost)"),
            }
            println!();
        }
    } else {
        let rng = Rng::with_seed(opt.seed);
        for (i, odds) in odds(
            opt.opponents,
            &players,
            &opt.board,
            deck,
            opt.permutations,
            RngAdapter(rng),
        )
        .into_iter()
        .enumerate()
        {
            println!(
                "player {}: win {:5.2}%, tie {:5.2}%, loss {:5.2}%",
                i + 1,
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
