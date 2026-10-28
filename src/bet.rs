/*!

  An abstraction for chips for betting.

*/

use std::{
    collections::HashMap,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
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

impl Bet {
    /// Returns the number of betting units
    pub fn units(&self) -> usize {
        self.units
    }
}

impl From<Bet> for usize {
    fn from(val: Bet) -> Self {
        val.units
    }
}

impl From<&Bet> for usize {
    fn from(val: &Bet) -> Self {
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

impl From<usize> for Bet {
    fn from(mut val: usize) -> Self {
        let mut units = 0;
        let mut chips = HashMap::new();

        for chip in [
            Chip::Thousand,
            Chip::FiveHundred,
            Chip::Hundred,
            Chip::TwentyFive,
            Chip::Five,
            Chip::One,
        ] {
            let count = val / chip.value();
            if count > 0 {
                chips.insert(chip, count);
                units += count * chip.value();
                val -= count * chip.value();
            }
        }

        Self { chips, units }
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

impl AddAssign for Bet {
    fn add_assign(&mut self, rhs: Self) {
        self.units += rhs.units;
        for (chip, count) in rhs.chips {
            *self.chips.entry(chip).or_insert(0) += count;
        }
    }
}

impl Sub for Bet {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let units = self.units - other.units;
        Bet::from(units)
    }
}

impl SubAssign for Bet {
    fn sub_assign(&mut self, rhs: Self) {
        let units = self.units - rhs.units;
        *self = Bet::from(units);
    }
}

/// Multiplication of a bet by a usize
impl Mul<usize> for Bet {
    type Output = Self;

    fn mul(mut self, rhs: usize) -> Bet {
        self.units *= rhs;
        for count in self.chips.values_mut() {
            *count *= rhs;
        }
        self
    }
}

/// Division of a bet by a usize
impl Div<usize> for Bet {
    type Output = Self;

    fn div(self, rhs: usize) -> Bet {
        let units = self.units / rhs;
        Bet::from(units)
    }
}

impl Sum for Bet {
    fn sum<I: Iterator<Item = Bet>>(iter: I) -> Self {
        iter.fold(Bet::default(), |acc, x| acc + x)
    }
}
