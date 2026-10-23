/*!

  An abstraction for chips for betting.

*/

use std::{
    collections::HashMap,
    ops::{Add, Mul, Sub},
};

/// A chip to place a bet with
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Copy)]
pub enum Chip {
    /// A chip worth 1 unit
    One,
    /// A chip worth 5 units
    Five,
    /// A chip worth 25 units
    TwentyFive,
    /// A chip worth 100 units
    Hundred,
    /// A chip worth 500 units
    FiveHundred,
    /// A chip worth 1000 units
    Thousand,
}

impl Chip {
    /// Get the value of the chip in units
    fn value(&self) -> usize {
        match self {
            Self::One => 1,
            Self::Five => 5,
            Self::TwentyFive => 25,
            Self::Hundred => 100,
            Self::FiveHundred => 500,
            Self::Thousand => 1000,
        }
    }
}

impl Add for Chip {
    type Output = Bet;

    fn add(self, other: Self) -> Bet {
        Bet::from(self) + Bet::from(other)
    }
}

/// A bet of one or more chips
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Bet {
    /// The chips in the bet
    chips: HashMap<Chip, usize>,
    /// The total value of the bet in units
    units: usize,
}

impl From<Chip> for Bet {
    fn from(chip: Chip) -> Self {
        Self {
            chips: HashMap::from([(chip, 1)]),
            units: chip.value(),
        }
    }
}

impl From<Bet> for usize {
    fn from(val: Bet) -> Self {
        val.units
    }
}

impl PartialOrd for Bet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.units.cmp(&other.units)
    }
}

impl Bet {
    /// Returns true if the bet is zero/empty
    pub fn is_empty(&self) -> bool {
        self.units == 0
    }
}

impl IntoIterator for Bet {
    type Item = Chip;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::new();
        for chip in [
            Chip::Thousand,
            Chip::FiveHundred,
            Chip::Hundred,
            Chip::TwentyFive,
            Chip::Five,
            Chip::One,
        ] {
            if let Some(&count) = self.chips.get(&chip) {
                for _ in 0..count {
                    vec.push(chip);
                }
            }
        }
        vec.into_iter()
    }
}

impl Add for Bet {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut chips = self.chips;
        let units = self.units + other.units;
        for (chip, count) in other.chips {
            *chips.entry(chip).or_insert(0) += count;
        }
        Self { chips, units }
    }
}

impl Sub for Bet {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut chips = self.chips;
        for (chip, count) in other.chips {
            *chips.entry(chip).or_insert(0) -= count;
        }
        let units = self.units - other.units;
        Self { chips, units }
    }
}

/// Multiplication of a bet by a usize
impl Mul<usize> for Bet {
    type Output = Self;

    fn mul(self, rhs: usize) -> Bet {
        let mut chips = self.chips;
        let units = self.units * rhs;
        for count in chips.values_mut() {
            *count *= rhs;
        }
        Self { chips, units }
    }
}
