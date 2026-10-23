/*!

  A shoe from which to deal cards

*/

use crate::card::{Card, Rank, Suit};
use rand::{rng, seq::SliceRandom};
use std::ops::Index;

/// A trait for card counting strategies
pub trait Counter {
    /// Returns the number of decks used in the game
    fn num_decks(&self) -> usize;

    /// Clears the counter
    fn clear(&mut self);

    /// Get the current count
    fn count(&self) -> f32;

    /// Update the count with a dealt card
    fn insert(&mut self, card: Card);
}

/// The high-low card counting strategy
pub struct HiLoCounter {
    running_count: i32,
    num_decks: usize,
}

impl HiLoCounter {
    /// Create a new high-low counter with the given number of decks
    pub fn new(num_decks: usize) -> Self {
        Self {
            running_count: 0,
            num_decks,
        }
    }
}

impl Counter for HiLoCounter {
    fn num_decks(&self) -> usize {
        self.num_decks
    }

    fn clear(&mut self) {
        self.running_count = 0;
    }

    fn count(&self) -> f32 {
        (self.running_count as f32) / (self.num_decks as f32)
    }

    fn insert(&mut self, card: Card) {
        self.running_count += card.count() as i32;
    }
}

/// A shoe of cards
pub struct Shoe {
    /// the cards
    cards: Vec<Card>,
    /// The running count
    counter: Box<dyn Counter>,
    /// the number of decks
    decks: usize,
}

impl Shoe {
    fn init_cards(num_decks: usize) -> Vec<Card> {
        let mut cards = Vec::new();
        for &s in &[Suit::Heart, Suit::Diamond, Suit::Club, Suit::Spade] {
            for &r in &[
                Rank::Ace,
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
            ] {
                for _ in 0..num_decks {
                    cards.push(Card::new(s, r));
                }
            }
        }
        cards
    }

    /// Create a new shoe with a custom counting strategy
    pub fn new_custom<C: Counter + 'static>(counter: C) -> Self {
        let decks = counter.num_decks();

        let cards = Self::init_cards(decks);

        let mut shoe = Self {
            cards,
            counter: Box::new(counter),
            decks,
        };
        shoe.shuffle();
        shoe
    }

    /// Create a new shoe using the high-low counting strategy
    pub fn new(decks: usize) -> Self {
        let counter = HiLoCounter::new(decks);
        Self::new_custom(counter)
    }
}

impl Index<usize> for Shoe {
    type Output = Card;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cards[index]
    }
}

impl Shoe {
    /// Returns the number of cards in the shoe
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// Returns true if the shoe is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Shuffle the shoe
    fn shuffle(&mut self) {
        let mut rng = rng();
        self.cards.shuffle(&mut rng);
    }

    /// Deal a card from the shoe
    pub fn deal(&mut self) -> Option<Card> {
        if let Some(card) = self.cards.pop() {
            self.counter.insert(card);
            Some(card)
        } else {
            None
        }
    }

    /// Returns the number of decks loaded into the shoe
    pub fn num_decks(&self) -> usize {
        self.decks
    }

    /// Return the running count
    pub fn running_count(&self) -> f32 {
        self.counter.count()
    }

    /// Returns how far the deck has been penetrated
    pub fn penetration(&self) -> f32 {
        1.0 - (self.cards.len() as f32) / ((self.decks * 52) as f32)
    }

    /// Forcibly reset the shoe
    pub fn reset(&mut self) {
        self.cards = Self::init_cards(self.decks);
        self.shuffle();
        self.counter.clear();
    }
}
