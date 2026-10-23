/*!

  A blackjack table

*/

use crate::{bet::Bet, card::Card, hand::Hand, shoe::Shoe};
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
pub struct Table {
    /// The dealer's hand
    dealer: Hand,
    /// The player hands
    player_hands: Vec<Hand>,
    /// The player bets
    _player_bets: Vec<Bet>,
    /// The shoe
    shoe: Shoe,
    /// The max deck penetration before reshuffle
    max_penetration: f32,
    /// The number of decks to play with
    num_decks: usize,
    /// State of the table
    state: TableState,
}

impl Table {
    /// Creates a new blackjack table with `num_decks` decks and `num_spots` bettings spots and `max_penetration` (from 0.0-1.0 before shuffling)
    pub fn new(num_decks: usize, num_spots: usize, max_penetration: f32) -> Self {
        // Uses hi-lo strategy
        let shoe = Shoe::new(num_decks);
        let player_hands = vec![Hand::default(); num_spots];
        let _player_bets = vec![Bet::default(); num_spots];
        let dealer = Hand::default();

        let max_penetration = max_penetration.clamp(0.0, 1.0);

        Self {
            dealer,
            player_hands,
            _player_bets,
            shoe,
            max_penetration,
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
        for player in &mut self.player_hands {
            *player = Hand::default();
        }
    }

    /// Clears all hands from the table
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    pub fn clear_hands(&mut self) {
        if self.state != TableState::Flipped {
            panic!("Cannot clear hands before dealer has flipped");
        }
        self.dealer = Hand::default();
        for player in &mut self.player_hands {
            *player = Hand::default();
        }
        self.state = TableState::Open;
    }

    /// Deal the initial hands for `k` players and the dealer. Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    pub fn deal(&mut self, k: usize) -> bool {
        if self.state != TableState::Open {
            panic!("Cannot deal when hands are currently on the table");
        }

        for player in &self.player_hands {
            debug_assert!(player.is_empty());
        }

        let reshuffle = self.shoe.penetration() > self.max_penetration;
        if reshuffle {
            self.shoe = Shoe::new(self.num_decks);
        }

        self.state = TableState::Dealt;

        for _ in 0..2 {
            for player in &mut self.player_hands.iter_mut().take(k) {
                player.insert(self.shoe.deal().unwrap());
            }
            self.dealer.insert(self.shoe.deal().unwrap());
        }

        reshuffle
    }

    /// Deal the initial hands for all player spots and the dealer. Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    pub fn deal_all(&mut self) -> bool {
        self.deal(self.player_hands.len())
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
    pub fn player_hand(&self, player: usize) -> &Hand {
        &self.player_hands[player]
    }

    /// Get the outcome for a player's hand
    pub fn get_outcome(&self, player: usize) -> Outcome {
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
        if self.state == TableState::Open {
            return None;
        }

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
