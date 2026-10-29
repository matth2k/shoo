use shoo::strategy::{Decision, Simulator, Strategy};
use shoo::{Bet, Card, Counter, Hand, HiLoCounter, Rank, table::Outcome};

pub struct SometimesHit;

impl Strategy for SometimesHit {
    fn bet(&self) -> Vec<Bet> {
        vec![Bet::from(shoo::Chip::Five); self.max_hands()]
    }

    fn decide(&self, hand: &Hand, upcard: Card) -> Decision {
        match hand.value() {
            Some(11) | Some(10) if !matches!(upcard.rank(), Rank::Ace | Rank::Ten) => {
                Decision::Double
            }
            Some(v) if v <= 11 => Decision::Hit,
            _ => Decision::Stand,
        }
    }

    fn process(&mut self, _hands: &[Hand], _dealer: &Hand, _outcomes: &[Outcome]) {}

    fn shoe_shuffled(&mut self) {}

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "SometimesHit".to_string()
    }
}

pub struct BasicStrategy {
    counter: Option<HiLoCounter>,
}

impl Strategy for BasicStrategy {
    fn bet(&self) -> Vec<Bet> {
        if self.counter.is_none() {
            return vec![Bet::from(shoo::Chip::Five); 1];
        }

        match self.counter.as_ref().unwrap().count() {
            c if c < 1.0 => vec![Bet::from(shoo::Chip::One); 1],
            c if c < 2.0 => vec![Bet::from(shoo::Chip::Five) * 4; 2],
            c if c < 3.0 => vec![Bet::from(shoo::Chip::Five) * 8; 2],
            c if c < 12.0 => vec![Bet::from(shoo::Chip::Five) * 16; 2],
            _ => vec![Bet::from(shoo::Chip::Five); 1],
        }
    }

    fn decide(&self, hand: &Hand, upcard: Card) -> Decision {
        if hand.value().is_none() {
            return Decision::Stand;
        }
        let val = hand.value().unwrap();
        if hand.is_soft() {
            match (val, upcard.values()[0]) {
                (13 | 14, 5 | 6) => Decision::Double,
                (13 | 14, _) => Decision::Hit,
                (15 | 16, 4..=6) => Decision::Double,
                (15 | 16, _) => Decision::Hit,
                (17, 1 | 10 | 9 | 8 | 7 | 2) => Decision::Hit,
                (17, 3..=6) => Decision::Double,
                (18, 2..=6) => Decision::Double,
                (18, 9 | 10 | 1) => Decision::Hit,
                (19, 6) => Decision::Double,
                _ => Decision::Stand,
            }
        } else if hand.is_pair() {
            match (val, upcard.values()[0]) {
                (6 | 4, 1 | 10 | 9 | 8) => Decision::Hit,
                (6 | 4, 2..=7) => Decision::Split,
                (8, 1 | 10 | 9 | 8 | 7 | 4 | 3 | 2) => Decision::Hit,
                (8, 5 | 6) => Decision::Split,
                (10, 2..=9) => Decision::Split,
                (10, 1 | 10) => Decision::Hit,
                (12, 1 | 10 | 9 | 8 | 7) if !hand.first().unwrap().is_ace() => Decision::Hit,
                (12, 2..=6) if !hand.first().unwrap().is_ace() => Decision::Split,
                (14, 2..=7) => Decision::Split,
                (14, 1 | 10 | 9 | 8) => Decision::Hit,
                (16, _) => Decision::Split,
                (18, 9 | 8 | 6 | 5 | 4 | 3 | 2) => Decision::Split,
                _ if hand.first().unwrap().is_ace() => Decision::Split,
                _ => Decision::Stand,
            }
        } else {
            match (val, upcard.values()[0]) {
                (12..=16, 1 | 10 | 9 | 8 | 7) => Decision::Hit,
                (12, 3 | 2) => Decision::Hit,
                (11, _) => Decision::Double,
                (10, 2..=9) => Decision::Double,
                (10, 1 | 10) => Decision::Hit,
                (9, 3..=6) => Decision::Double,
                (v, _) if v <= 9 => Decision::Hit,
                _ => Decision::Stand,
            }
        }
    }

    fn process(&mut self, hands: &[Hand], dealer: &Hand, _outcomes: &[Outcome]) {
        for hand in hands {
            for card in hand.into_iter() {
                if let Some(counter) = &mut self.counter {
                    counter.insert(card);
                }
            }
        }
        for card in dealer.into_iter() {
            if let Some(counter) = &mut self.counter {
                counter.insert(card);
            }
        }
    }

    fn shoe_shuffled(&mut self) {
        if let Some(counter) = &mut self.counter {
            counter.clear();
        }
    }

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "BasicStrategy".to_string()
    }
}

impl BasicStrategy {
    pub fn new(num_decks: Option<usize>) -> Self {
        Self {
            counter: num_decks.map(HiLoCounter::new),
        }
    }
}

pub struct SometimesHitTwo;

impl Strategy for SometimesHitTwo {
    fn bet(&self) -> Vec<Bet> {
        vec![Bet::from(shoo::Chip::Five); self.max_hands()]
    }

    fn decide(&self, hand: &Hand, upcard: Card) -> Decision {
        match hand.value() {
            Some(11) | Some(10) if !matches!(upcard.rank(), Rank::Ace | Rank::Ten) => {
                Decision::Double
            }
            Some(v) if v <= 16 && hand.is_pair() && upcard.rank() <= Rank::Six => Decision::Split,
            Some(v) if v <= 11 => Decision::Hit,
            Some(v) if v <= 16 && upcard.rank() >= Rank::Seven => Decision::Hit,
            _ => Decision::Stand,
        }
    }

    fn process(&mut self, _hands: &[Hand], _dealer: &Hand, _outcomes: &[Outcome]) {}

    fn shoe_shuffled(&mut self) {}

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "SometimesHitTwo".to_string()
    }
}

pub struct AlwaysStand;

impl Strategy for AlwaysStand {
    fn bet(&self) -> Vec<Bet> {
        vec![Bet::from(shoo::Chip::Five); self.max_hands()]
    }

    fn decide(&self, _hand: &Hand, _upcard: Card) -> Decision {
        Decision::Stand
    }

    fn process(&mut self, _hands: &[Hand], _dealer: &Hand, _outcomes: &[Outcome]) {}

    fn shoe_shuffled(&mut self) {}

    fn max_hands(&self) -> usize {
        2
    }

    fn name(&self) -> String {
        "AlwaysStand".to_string()
    }
}

fn main() -> std::io::Result<()> {
    let bankroll: Bet = Bet::from(100000);
    let num_decks = 2;
    let mut trials = Vec::new();
    for _ in 0..20 {
        let mut sim = Simulator::new(SometimesHitTwo, bankroll.clone(), num_decks, 0.66);
        sim.insert(BasicStrategy::new(Some(num_decks)), bankroll.clone());

        let res = sim.simulate(100000);

        trials.push(res);
    }

    // Pad all trials to the same length of longest simulation.
    let maxlen = trials.iter().map(|v| v.len()).max().unwrap_or(0);
    for trial in trials.iter_mut() {
        while trial.len() < maxlen {
            trial.push(trial.last().unwrap().clone());
        }
    }

    let mut wtr = csv::Writer::from_path("gambling_trials.csv")?;

    let mut headers = vec!["Iter".to_string()];
    for i in 0..trials.len() {
        headers.push(format!("Trial {}", i + 1));
    }
    wtr.write_record(&headers)?;

    for i in 0..maxlen {
        let mut record = vec![i.to_string()];
        for trial in &trials {
            let diff = trial[i][1] as isize - trial[i][0] as isize;
            record.push(diff.to_string());
        }
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}
