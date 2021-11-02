# odd ♠️

_a rusty Texas Hold'em poker odds simulator_

## Installation

```sh
brew install rustup
git clone git@github.com:ocisly/odd.git
cd odd
rustup -V
cargo install --path .
```

## Usage

```sh
odd <hole-cards>... --opponents <opponents> --permutations <permutations> --seed <seed>
```

When all the players' hole cards and all five community cards are known, `odd`
detects what hand each player holds and determines the winners:

```sh
odd Ad Ks 8h 8s --board 8c Ah As 9c 9h
```

```
player 1 was dealt: A♦️ K♠️
player 2 was dealt: 8♥️ 8♠️
flop: 8♣️ A♥️ A♠️
turn: 9♣️
river: 9♥️

43 cards remain.

player 1 has Full House, As full of 9s: A♦️ A♥️ A♠️ 9♣️ 9♥️ (winner)
player 2 has Full House, 8s full of As: 8♥️ 8♠️ 8♣️ A♥️ A♠️ (lost)
```

Otherwise, `odd` estimates the odds of winning for each player by generating a
configurable number of random deck shuffles to simulate a range of possible
scenarios:

```sh
odd Ad Ks 8h 8s --board 8c Ah As 9c
```

```
player 1 was dealt: A♦️ K♠️
player 2 was dealt: 8♥️ 8♠️
flop: 8♣️ A♥️ A♠️
turn: 9♣️

44 cards remain.

player 1: win 15.90%, tie  0.00%, loss 84.10%
player 2: win 84.10%, tie  0.00%, loss 15.90%
```

Additional opponents with unknown cards can be specified:

```sh
odd As Ah -o 1
```

```
player 1 was dealt: A♠️ A♥️

50 cards remain.

player 1: win 82.10%, tie  0.54%, loss 17.36%
player 2: win 17.36%, tie  0.54%, loss 82.10%
```
