/*!

  Simulate a blackjack strategy

*/

use crate::table::{Outcome, Table};
use crate::{Bet, Card, Hand};

/// The action a blackjack player can take
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Decision {
    /// Hit
    Hit,
    /// Stand
    Stand,
    /// Double down
    Double,
    /// Split
    Split,
}

/// Decisions needed to play blackjack
pub trait Strategy {
    /// Place a bet in the current context
    fn bet(&self) -> Bet;

    /// Make a choice based on the hand and upcard
    fn decide(&self, hand: &Hand, upcard: Card) -> Decision;

    /// Process the last outcome of the hand
    fn process(&mut self, hand: &Hand, dealer: &Hand, outcome: Outcome);
}

/// Run a blackjack strategy
pub struct Simulator {
    /// The strategy to run
    strategy: Box<dyn Strategy>,
    /// The bankroll of the player
    bankroll: Bet,
    /// The blackjack table to play on
    table: Table,
}

impl Simulator {
    /// Create a new simulator with the given strategy and bankroll
    pub fn new<S: Strategy + 'static>(
        strategy: S,
        bankroll: Bet,
        num_decks: usize,
        max_spots: usize,
        max_penetration: f32,
    ) -> Self {
        Self {
            strategy: Box::new(strategy),
            bankroll,
            table: Table::new(num_decks, max_spots, max_penetration),
        }
    }

    fn process_outcomes(&mut self, bets: Vec<Bet>, outcomes: &Vec<Outcome>) {
        for (i, outcome) in outcomes.iter().copied().enumerate() {
            self.strategy
                .process(self.table.player_hand(i), self.table.dealer_hand(), outcome);
        }
        self.update_bankroll(bets, outcomes);
        self.table.clear_hands();
    }

    /// Add or subtract for the bankrolls
    fn update_bankroll(&mut self, bets: Vec<Bet>, outcomes: &Vec<Outcome>) {
        for (bet, &outcome) in bets.into_iter().zip(outcomes) {
            match outcome {
                Outcome::Win => self.bankroll = self.bankroll.clone() + bet,
                Outcome::Lose => self.bankroll = self.bankroll.clone() - bet,
                Outcome::Blackjack => {
                    self.bankroll = self.bankroll.clone() + bet.clone() + (bet / 2)
                }
                Outcome::Push => {}
            }
        }
    }

    /// Run a single iteration with a set of bets. Returns the outcomes of the hands.
    fn run_hand(&mut self, mut bets: Vec<Bet>) -> Vec<Outcome> {
        let k = bets.len();

        self.table.deal(k);

        if self.table.peek() {
            let outcomes: Vec<_> = (0..k).map(|i| self.table.get_outcome(i)).collect();
            self.process_outcomes(bets, &outcomes);
            return outcomes;
        }
        let mut outcomes = vec![Outcome::Push; k];

        let upcard = self.table.upcard().unwrap();
        for i in 0..k {
            while let hand = self.table.player_hand(i)
                && !hand.busted()
            {
                match self.strategy.decide(hand, upcard) {
                    Decision::Double
                        if self.bankroll.units()
                            >= bets[i].units() + bets.iter().map(|b| b.units()).sum::<usize>() =>
                    {
                        bets[i] = bets[i].clone() * 2;
                        self.table.player_hit(i);
                        if self.table.player_hand(i).busted() {
                            outcomes[i] = self.table.get_outcome(i);
                        }
                    }
                    Decision::Hit | Decision::Double => {
                        self.table.player_hit(i);
                        if self.table.player_hand(i).busted() {
                            outcomes[i] = self.table.get_outcome(i);
                        }
                        break;
                    }
                    Decision::Stand => break,
                    Decision::Split => {
                        todo!("Implement split");
                    }
                }
            }
        }

        if outcomes.iter().any(|o| *o != Outcome::Lose) {
            while self.table.dealer_value().is_some_and(|v| v < 17) {
                self.table.dealer_hit();
            }
        }

        self.table.flip_hole();

        for (i, outcome) in outcomes.iter_mut().enumerate() {
            *outcome = self.table.get_outcome(i);
        }

        self.process_outcomes(bets, &outcomes);
        outcomes
    }

    /// Simulate the strategy for a number of iterations, playing a certain number of hands.
    /// Returns the bankroll state at each iter.
    pub fn simulate(&mut self, iter_max: usize, hands_per_iter: usize) -> Vec<usize> {
        let mut hist = Vec::new();
        for _ in 0..iter_max {
            let bets = vec![self.strategy.bet(); hands_per_iter];
            let amt: Bet = bets.iter().cloned().sum();

            if amt > self.bankroll {
                return hist;
            }

            self.run_hand(bets);
            hist.push(self.bankroll.units());
        }

        hist
    }
}
