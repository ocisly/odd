# REST API

## Installation

```sh
brew install rustup
git clone git@github.com:ocisly/odd.git
cd odd
rustup -V
cargo install --path web
```

## Usage

Run the server

```sh
odd-web
```

Query it

```sh
curl --silent --data '{
  "players": [["As", "Kh"], ["Ac", "Ad"]],
  "board": ["5c", "6c", "7c"]
}' localhost:8080/run | jq
```

```json
{
  "cards_remaining": 45,
  "odds": [
    {
      "distribution": {
        "Flush": "3.35%",
        "High Card": "38.01%",
        "Pair": "46.01%",
        "Straight": "4.46%",
        "Straight Flush": "0.29%",
        "Three of a Kind": "1.27%",
        "Two Pair": "6.62%"
      },
      "loss": "96.93%",
      "tie": "2.95%",
      "win": "0.12%"
    },
    {
      "distribution": {
        "Flush": "35.95%",
        "Full House": "1.85%",
        "Pair": "27.70%",
        "Straight": "2.67%",
        "Straight Flush": "0.29%",
        "Three of a Kind": "2.57%",
        "Two Pair": "28.97%"
      },
      "loss": "0.12%",
      "tie": "2.95%",
      "win": "96.93%"
    }
  ]
}
```
