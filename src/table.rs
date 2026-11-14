/*!

  A blackjack table

*/

use crate::{card::Card, hand::Hand, shoe::Shoe};
use rand::rng;
use rand_distr::{Distribution, Normal};
use std::fmt::Display;

/// Represents the outcome of a hand
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Outcome {
    /// Pays 3:2 or 6:5
    Blackjack,
    /// Pays 1:1
    Win,
    /// A loss
    Lose,
    /// A push
    Push,
    /// A surrendered hand
    Forfeit,
}

/// Represents the state of the table
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum TableState {
    /// Open for bets
    Open,
    /// Initial cards dealt
    Dealt,
    /// The dealer flipped
    Flipped,
}

/// A blackjack table
#[derive(Debug, Clone)]
pub struct Table {
    /// The dealer's hand
    dealer: Hand,
    /// The player hands
    player_hands: Vec<Hand>,
    /// The shoe
    shoe: Shoe,
    /// The max deck penetration before reshuffle
    max_penetration: f32,
    /// The current deck penetration
    penetration: f32,
    /// Simulate randomness of cutting the cards
    pen_dev: Option<f32>,
    /// The number of decks to play with
    num_decks: usize,
    /// State of the table
    state: TableState,
}

impl Table {
    fn gen_pen(max_pen: f32, pen_dev: Option<f32>) -> f32 {
        let val = if let Some(dev) = pen_dev {
            let mut rng = rng();
            let normal = Normal::new(max_pen, dev).expect("Failed to create normal distribution");
            normal.sample(&mut rng)
        } else {
            max_pen
        };
        val.clamp(0.0, 0.85)
    }

    /// Creates a new blackjack table with a certain number of decks, penetration, and penetration variance
    pub fn new(num_decks: usize, max_penetration: f32, pen_dev: Option<f32>) -> Self {
        // Uses hi-lo strategy
        let shoe = Shoe::new(num_decks);
        let player_hands = Vec::new();
        let dealer = Hand::default();

        let penetration = Self::gen_pen(max_penetration, pen_dev);

        Self {
            dealer,
            player_hands,
            shoe,
            max_penetration,
            penetration,
            pen_dev,
            num_decks,
            state: TableState::Open,
        }
    }

    /// Resets the table
    ///
    /// # Panics
    /// Panics if there are cards currently dealt
    pub fn reset(&mut self) {
        if self.state != TableState::Open {
            panic!("Cannot reset table while cards are dealt");
        }

        self.shoe = Shoe::new(self.num_decks);
        self.dealer = Hand::default();
        self.player_hands.clear();
        self.penetration = Self::gen_pen(self.max_penetration, self.pen_dev);
    }

    /// Clears all hands from the table
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn clear_hands(&mut self) {
        if self.state == TableState::Dealt {
            panic!("Cannot clear hands before dealer has flipped");
        }
        self.dealer = Hand::default();
        self.player_hands.clear();
        self.state = TableState::Open;
    }

    /// Mark a betting spot as active for this round
    ///
    /// # Panics
    /// Panics if there are cards currently dealt
    pub fn get_spot(&mut self) -> usize {
        if self.state != TableState::Open {
            panic!("Cannot take spot when hands are currently on the table");
        }
        self.player_hands.push(Hand::default());
        self.player_hands.len() - 1
    }

    /// Deal the active betting spots and the dealer.
    /// Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    pub fn deal(&mut self) -> bool {
        if self.state != TableState::Open {
            panic!("Cannot deal when hands are currently on the table");
        }

        for player in &self.player_hands {
            debug_assert!(player.is_empty());
        }

        let reshuffle = self.shoe.penetration() > self.penetration;
        if reshuffle {
            self.penetration = Self::gen_pen(self.max_penetration, self.pen_dev);
            self.shoe = Shoe::new(self.num_decks);
        }

        self.state = TableState::Dealt;

        for _ in 0..2 {
            for player in &mut self.player_hands {
                player.insert(self.shoe.deal().unwrap());
            }
            self.dealer.insert(self.shoe.deal().unwrap());
        }

        reshuffle
    }

    /// Returns true if the dealer has blackjack
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn peek(&mut self) -> bool {
        if self.state != TableState::Dealt {
            panic!("Cannot peek when no cards are dealt");
        }

        if self.dealer.blackjack() {
            self.state = TableState::Flipped;
            true
        } else {
            false
        }
    }

    /// Returns the upcard of the dealer
    pub fn upcard(&self) -> Option<Card> {
        self.dealer.first()
    }

    /// Returns a reference to the hand of player `player`
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn player_hand(&mut self, player: usize) -> &Hand {
        let hand: &mut Hand = &mut self.player_hands[player];

        // Deal the card after split
        if self.state == TableState::Dealt && hand.len() == 1 {
            hand.insert(self.shoe.deal().unwrap());
        }
        hand
    }

    /// Returns a reference to the hand of the dealer
    ///
    /// # Panics
    /// Panics if the dealer has not flipped their hole card yet
    pub fn dealer_hand(&self) -> &Hand {
        if self.state == TableState::Dealt {
            panic!("Cannot get dealer hand when a hand is in progress");
        }
        &self.dealer
    }

    /// Get the outcome for a player's hand
    pub fn get_outcome(&mut self, player: usize) -> Outcome {
        debug_assert!(if let Some(v) = self.dealer.value()
            && v < 17
        {
            self.player_hands[player].blackjack() || self.player_hands[player].busted()
        } else {
            true
        });

        if self.dealer.blackjack() {
            return if self.player_hand(player).blackjack() {
                Outcome::Push
            } else {
                Outcome::Lose
            };
        }

        if self.player_hand(player).blackjack() {
            return Outcome::Blackjack;
        }

        if self.player_hand(player).busted() {
            return Outcome::Lose;
        }

        if self.dealer.busted() {
            return Outcome::Win;
        }

        match (
            self.player_hand(player).value().unwrap(),
            self.dealer.value().unwrap(),
        ) {
            (p, d) if p > d => Outcome::Win,
            (p, d) if p < d => Outcome::Lose,
            _ => Outcome::Push,
        }
    }

    /// Deal a player an additional card. Returns true if the player busted.
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn player_hit(&mut self, player: usize) -> bool {
        if self.state != TableState::Dealt {
            panic!("Cannot hit when no cards are dealt");
        }

        if self.player_hand(player).busted() {
            return true;
        }

        self.player_hands[player].insert(self.shoe.deal().unwrap());

        self.player_hand(player).busted()
    }

    /// Returns the new hands after a player split
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn player_split(&mut self, player: usize) -> Option<(usize, usize)> {
        if self.state != TableState::Dealt {
            panic!("Cannot split when no cards are dealt");
        }

        if !self.player_hand(player).is_pair() {
            return None;
        }

        let other_spot = self.player_hands.len();

        let cards: Vec<Card> = self.player_hands[player]
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        let first = cards[0];
        let second = cards[1];
        self.player_hands[player] = Hand::default();
        self.player_hands.push(Hand::default());
        self.player_hands[player].insert(first);
        self.player_hands[other_spot].insert(second);

        Some((player, other_spot))
    }

    /// The dealer hits. Also flips if not done.
    /// Returns true if the dealer busted.
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn dealer_hit(&mut self) -> bool {
        if self.state == TableState::Open {
            panic!("Cannot hit dealer when no cards are dealt");
        }

        if self.dealer.busted() {
            return true;
        }

        self.dealer.insert(self.shoe.deal().unwrap());

        self.state = TableState::Flipped;
        self.dealer.busted()
    }

    /// The dealer value
    pub fn dealer_value(&self) -> Option<u8> {
        self.dealer.value()
    }

    /// An iterator over the player hands
    pub fn player_hands(&self) -> impl Iterator<Item = &Hand> {
        self.player_hands.iter()
    }

    /// Flip the dealer's hole card
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn flip_hole(&mut self) {
        if self.state == TableState::Open {
            panic!("Cannot flip hole card when no cards are dealt");
        }
        self.state = TableState::Flipped;
    }

    /// Returns the number of active spots at the table
    pub fn num_spots(&self) -> usize {
        self.player_hands.len()
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\n\n")?;
        writeln!(
            f,
            "Dealer: {}    {} {}",
            self.dealer,
            self.shoe.running_count(),
            self.shoe.penetration()
        )?;
        for (i, player) in self.player_hands().enumerate() {
            writeln!(f, "Player {}: {}", i + 1, player)?;
        }
        Ok(())
    }
}
