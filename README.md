![](https://github.com/matth2k/shoo/actions/workflows/rust.yml/badge.svg)
[![Docs](https://img.shields.io/badge/docs-github--pages-blue)](https://matth2k.github.io/shoo/)
[![crates.io](https://img.shields.io/badge/crates.io-github--pages-blue)](https://crates.io/crates/shoo)

# Shoo: A Library for Blackjack training

## Description

A Rust library to assist in creating your own blackjack trainer or game.

## Getting Started

Below is a minimal example to get you started:

```rust
use shoo::{Card, Hand, Rank, Suit};

#[test]
fn test_hand() {
    let mut hand = Hand::default();
    hand.insert(Card::new(Suit::Spade, Rank::Ace));
    hand.insert(Card::new(Suit::Heart, Rank::King));
    assert!(hand.value().is_some());
    assert!(hand.blackjack());
}

```

## Building a Strategy Simulator

With the `Strategy` trait and `Simulator` struct, you can test and compare how strategies perform at the blackjack table. Let's try two different strategies: `AlwaysStand` and `SometimesHit`

```rust
pub struct AlwaysStand;

impl Strategy for AlwaysStand {
    fn bet(&self) -> Vec<Bet> {
        vec![Bet::from(shoo::Chip::Five); self.max_hands()]
    }

    fn decide(&self, _hand: &Hand, _upcard: Card) -> Decision {
        Decision::Stand
    }

    fn process(&mut self, _hand: &Hand, _dealer: &Hand, _outcome: Outcome) {}

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "AlwaysStand".to_string()
    }
}
```

A better strategy would use hitting, splitting, doubling of course. Here is something that is vaguely close to basic strategy without counting:

```rust
pub struct SometimesHit;

impl Strategy for SometimesHit {
    fn bet(&self) -> Vec<Bet> {
        vec![Bet::from(shoo::Chip::Five); self.max_hands()]
    }

    fn decide(&self, hand: &Hand, upcard: Card) -> Decision {
        match hand.value() {
            Some(11) | Some(10) if !matches!(upcard.rank(), shoo::Rank::Ace | shoo::Rank::Ten) => {
                Decision::Double
            }
            Some(v) if v <= 16 && hand.pairs() && upcard.rank() <= shoo::Rank::Six => {
                Decision::Split
            }
            Some(v) if v <= 11 => Decision::Hit,
            Some(v) if v <= 16 && upcard.rank() >= shoo::Rank::Seven => Decision::Hit,
            _ => Decision::Stand,
        }
    }

    fn process(&mut self, _hand: &Hand, _dealer: &Hand, _outcome: Outcome) {}

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "SometimesHit".to_string()
    }
}
```

If you want to simulate advantage play, a.k.a. card counting, you should add that within the `process()` hook. After every hand, the strategy can inspect the winning/losing hands and the dealer hand. Here you would consume the individual cards and add them to the running count. It is as simple as adding a card counter member field. You can even vary how many hands you want to play in `bet()`. I will leave those strategy implementations as an exercise to the reader. Lastly, you can simulate these strategies at a table:

```rust
// Bankroll of 10k units
let bankroll: Bet = Bet::from(10000);
// Create a simulator with an inital strategy, bankroll, number of decks, and deck penetration
let mut simulator = Simulator::new(AlwaysStand, bankroll.clone(), 6, 0.5);
// Add a second strategy with its own bankroll
sim.insert(SometimesHitTwo, bankroll);

// Run a simulation with max number of games
let res = sim.simulate(50000);

// res[t][i] is the bankroll state of strategy `i` at iteration `t`
```

## Limitations

Some features of Blackjack rulesets are not modeled by this application. Two things that come to mind are insurance bets, resplitting aces, and hit or stand on soft 17. This library was programmed with Pennsylvania Blackjack in mind, but rule difference should eventually be modeled.
