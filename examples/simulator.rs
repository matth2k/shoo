use shoo::strategy::{Decision, Simulator, Strategy};
use shoo::{Bet, Card, Hand, table::Outcome};

pub struct AlwaysStand;

impl Strategy for AlwaysStand {
    fn bet(&self) -> Bet {
        Bet::from(shoo::Chip::Five)
    }

    fn decide(&self, hand: &Hand, _upcard: Card) -> Decision {
        if let Some(val) = hand.value()
            && val < 12
        {
            Decision::Hit
        } else {
            Decision::Stand
        }
    }

    fn process(&mut self, _hand: &Hand, _dealer: &Hand, _outcome: Outcome) {}
}

fn main() -> std::io::Result<()> {
    let mut bankroll = Bet::default();
    for _ in 0..100 {
        bankroll = bankroll + Bet::from(shoo::Chip::Five);
    }
    let mut sim = Simulator::new(AlwaysStand, bankroll, 2, 6, 0.5);

    let res = sim.simulate(50, 2);

    eprintln!("Results: {:?}", res);

    Ok(())
}
