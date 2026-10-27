use shoo::strategy::{Decision, Simulator, Strategy};
use shoo::{Bet, Card, Hand, table::Outcome};

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
            Some(v) if v <= 11 => Decision::Hit,
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

fn main() -> std::io::Result<()> {
    let bankroll: Bet = Bet::from(10000);
    let mut trials = Vec::new();
    for _ in 0..20 {
        let mut sim = Simulator::new(SometimesHit, bankroll.clone(), 6, 0.5);
        sim.insert(SometimesHit, bankroll.clone());

        let res = sim.simulate(50000);

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
