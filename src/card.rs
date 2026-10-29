/*!

  Cards

*/

use std::fmt::Display;

/// Suit of a card
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Suit {
    /// Heart
    Heart,
    /// Diamond
    Diamond,
    /// Club
    Club,
    /// Spade
    Spade,
}

impl Display for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Heart => write!(f, "♡"),
            Self::Diamond => write!(f, "♢"),
            Self::Club => write!(f, "♧"),
            Self::Spade => write!(f, "♤"),
        }
    }
}

/// Rank of a card
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Rank {
    /// 2
    Two,
    /// 3
    Three,
    /// 4
    Four,
    /// 5
    Five,
    /// 6
    Six,
    /// 7
    Seven,
    /// 8
    Eight,
    /// 9
    Nine,
    /// 10
    Ten,
    /// 10
    Jack,
    /// 10
    Queen,
    /// 10
    King,
    /// 11/1
    Ace,
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ace => write!(f, "A"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::Seven => write!(f, "7"),
            Self::Eight => write!(f, "8"),
            Self::Nine => write!(f, "9"),
            Self::Ten => write!(f, "10"),
            Self::Jack => write!(f, "J"),
            Self::Queen => write!(f, "Q"),
            Self::King => write!(f, "K"),
        }
    }
}

/// A playing card
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
    }
}

impl Card {
    /// Create a new card
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    /// Get the suit of the card
    pub fn suit(&self) -> Suit {
        self.suit
    }

    /// Get the rank of the card
    pub fn rank(&self) -> Rank {
        self.rank
    }

    /// Get the value(s) of the card ([1, 11] for an ace, [2] for two, etc..)
    pub fn values(&self) -> &[u8] {
        match self.rank {
            Rank::Ace => &[1, 11],
            Rank::Two => &[2],
            Rank::Three => &[3],
            Rank::Four => &[4],
            Rank::Five => &[5],
            Rank::Six => &[6],
            Rank::Seven => &[7],
            Rank::Eight => &[8],
            Rank::Nine => &[9],
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King => &[10],
        }
    }

    /// Return the hi-lo count (+1, 0, -1) of a card
    pub fn count(&self) -> i8 {
        match self.rank {
            Rank::Two | Rank::Three | Rank::Four | Rank::Five | Rank::Six => 1,
            Rank::Seven | Rank::Eight | Rank::Nine => 0,
            Rank::Ten | Rank::Jack | Rank::Queen | Rank::King | Rank::Ace => -1,
        }
    }

    /// Returns true if card is an ace
    pub fn is_ace(&self) -> bool {
        self.rank == Rank::Ace
    }
}
