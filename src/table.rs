/*!

  A blackjack table

*/

use crate::{card::Card, hand::Hand, shoe::Shoe};
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
        let dealer = Hand::default();

        let max_penetration = max_penetration.clamp(0.0, 1.0);

        Self {
            dealer,
            player_hands,
            shoe,
            max_penetration,
            num_decks,
            state: TableState::Open,
        }
    }

    /// Creates a new blackjack table at the current state but with a different number of spots
    pub fn new_like(&self, num_spots: usize) -> Self {
        let mut other = self.clone();
        for _ in 0..num_spots {
            other.player_hands.push(Hand::default());
        }
        other
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
        if self.state == TableState::Dealt {
            panic!("Cannot clear hands before dealer has flipped");
        }
        self.dealer = Hand::default();
        for player in &mut self.player_hands {
            *player = Hand::default();
        }
        self.state = TableState::Open;
    }

    /// Deal the betting spots in `spots` and the dealer.
    /// Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    pub fn deal(&mut self, spots: &[usize]) -> bool {
        if self.state != TableState::Open {
            panic!("Cannot deal when hands are currently on the table");
        }

        let mut spots = spots.to_vec();
        spots.sort();
        spots.dedup();

        for player in &self.player_hands {
            debug_assert!(player.is_empty());
        }

        let reshuffle = self.shoe.penetration() > self.max_penetration;
        if reshuffle {
            self.shoe = Shoe::new(self.num_decks);
        }

        self.state = TableState::Dealt;

        for _ in 0..2 {
            for &player in &spots {
                self.player_hands[player].insert(self.shoe.deal().unwrap());
            }
            self.dealer.insert(self.shoe.deal().unwrap());
        }

        reshuffle
    }

    /// Deal the initial hands for the first `k` spots and the dealer.
    /// Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    /// Panics if k is out of range
    pub fn deal_up_to(&mut self, k: usize) -> bool {
        let all = (0..k).collect::<Vec<_>>();
        self.deal(&all)
    }

    /// Deal the initial hands for all player spots and the dealer.
    /// Returns true if dealing prompted the shoe to reshuffle.
    ///
    /// # Panics
    /// Panics if there are already cards dealt on the table
    pub fn deal_all(&mut self) -> bool {
        self.deal_up_to(self.player_hands.len())
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

    /// Returns the new hands after a player split
    ///
    /// # Panics
    /// Panics if there are no cards currently dealt
    /// Panics if the table runs out of spots
    pub fn player_split(&mut self, player: usize) -> Option<(usize, usize)> {
        if self.state != TableState::Dealt {
            panic!("Cannot split when no cards are dealt");
        }

        if !self.player_hand(player).is_pair() {
            return None;
        }

        let mut other_spot = player + 1;
        while !self.player_hands[other_spot].is_empty() {
            other_spot += 1;
        }

        let cards: Vec<Card> = self.player_hands[player]
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        let first = cards[0];
        let second = cards[1];
        self.player_hands[player] = Hand::default();
        self.player_hands[other_spot] = Hand::default();
        self.player_hands[player].insert(first);
        self.player_hands[player].insert(self.shoe.deal().unwrap());
        self.player_hands[other_spot].insert(second);
        self.player_hands[other_spot].insert(self.shoe.deal().unwrap());

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

    /// Returns the number of spots at the table
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
