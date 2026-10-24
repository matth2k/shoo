use shoo::strategy::{Decision, Simulator, Strategy};
use shoo::{Bet, Card, Hand, table::Outcome};

pub struct AlwaysStand;

impl Strategy for AlwaysStand {
    fn bet(&self) -> Bet {
        Bet::from(shoo::Chip::Five)
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
}

fn main() -> std::io::Result<()> {
    let bankroll: Bet = Bet::from(1000);
    let mut trials = Vec::new();
    for _ in 0..20 {
        let mut sim = Simulator::new(AlwaysStand, bankroll.clone(), 2, 6, 0.5);

        let res = sim.simulate(5000, 2);

        trials.push(res);
    }

    let maxlen = trials.iter().map(|v| v.len()).max().unwrap_or(0);
    for trial in trials.iter_mut() {
        while trial.len() < maxlen {
            trial.push(*trial.last().unwrap());
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
            record.push(trial[i].to_string());
        }
        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}
