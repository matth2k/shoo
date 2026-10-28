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
    /// Place a series of bets/hands in the current context
    fn bet(&self) -> Vec<Bet>;

    /// Make a choice based on the hand and upcard
    fn decide(&self, hand: &Hand, upcard: Card) -> Decision;

    /// Process the last outcome of the hand
    fn process(&mut self, hand: &Hand, dealer: &Hand, outcome: Outcome);

    /// Max number of hands this strategy ever uses
    fn max_hands(&self) -> usize;

    /// The name of this strategy
    fn name(&self) -> String;
}

/// Run a blackjack strategy
pub struct Simulator {
    /// The strategies to run
    strategies: Vec<Box<dyn Strategy>>,
    /// The bankrolls per stategy
    bankrolls: Vec<Bet>,
    /// The blackjack table to play on
    table: Table,
}

impl Simulator {
    /// Create a new simulator with the given strategy and bankroll
    pub fn new<S: Strategy + 'static>(
        strategy: S,
        bankroll: Bet,
        num_decks: usize,
        max_penetration: f32,
    ) -> Self {
        let bankrolls = vec![bankroll];
        let table = Table::new(num_decks, strategy.max_hands() * 2, max_penetration);
        let boxed: Box<dyn Strategy> = Box::new(strategy);
        let strategies = vec![boxed];
        Self {
            strategies,
            bankrolls,
            table,
        }
    }

    /// Add an additional strategy to the simulator
    pub fn insert<S: Strategy + 'static>(&mut self, strategy: S, bankroll: Bet) {
        self.bankrolls.push(bankroll);
        self.table = self
            .table
            .new_like(self.table.num_spots() + strategy.max_hands() * 2);
        let boxed: Box<dyn Strategy> = Box::new(strategy);
        self.strategies.push(boxed);
    }

    fn process_outcomes(&mut self, bets: Vec<(usize, Bet)>, outcomes: &Vec<Outcome>) {
        for (i, ((strat, _bet), &outcome)) in bets.iter().zip(outcomes).enumerate() {
            self.strategies[*strat].process(
                self.table.player_hand(i),
                self.table.dealer_hand(),
                outcome,
            );
        }
        self.update_bankroll(bets, outcomes);
        self.table.clear_hands();
    }

    /// Add or subtract for the bankrolls
    fn update_bankroll(&mut self, bets: Vec<(usize, Bet)>, outcomes: &Vec<Outcome>) {
        for ((strat, bet), &outcome) in bets.into_iter().zip(outcomes) {
            let double = bet.clone() * 2;
            match outcome {
                Outcome::Win => self.bankrolls[strat] += double,
                Outcome::Lose => {}
                Outcome::Blackjack => self.bankrolls[strat] += double + (bet / 2),
                Outcome::Push => self.bankrolls[strat] += bet,
            }
        }
    }

    /// Run a single iteration with a set of bets. Returns the outcomes of the hands.
    fn run_hand(&mut self, bets: Vec<(usize, Bet)>) -> Vec<Outcome> {
        if bets.is_empty() {
            return Vec::new();
        }

        let k = bets.len();
        self.table.deal_up_to(k);

        if self.table.peek() {
            let outcomes: Vec<_> = (0..k).map(|i| self.table.get_outcome(i)).collect();
            self.process_outcomes(bets, &outcomes);
            return outcomes;
        }
        let mut outcomes = vec![Outcome::Push; k];

        let upcard = self.table.upcard().unwrap();

        // Allocate each hand in order to a spot
        let mut bets: Vec<_> = bets.into_iter().enumerate().collect();
        let mut processed = Vec::new();

        for _ in 0..4 {
            let mut split_hands = Vec::new();
            for (i, (strat, bet)) in bets.iter_mut() {
                let i = *i;
                while let hand = self.table.player_hand(i)
                    && !hand.busted()
                {
                    match self.strategies[*strat].decide(hand, upcard) {
                        Decision::Double if self.bankrolls[*strat].units() >= bet.units() => {
                            self.bankrolls[*strat] -= bet.clone();
                            *bet = bet.clone() * 2;
                            self.table.player_hit(i);
                            if self.table.player_hand(i).busted() {
                                outcomes[i] = self.table.get_outcome(i);
                            }
                            break;
                        }
                        Decision::Split
                            if self.bankrolls[*strat].units() >= bet.units()
                                && let Some((_og, b)) = self.table.player_split(i) =>
                        {
                            self.bankrolls[*strat] -= bet.clone();
                            let new_bet = (b, (*strat, bet.clone()));
                            split_hands.push(new_bet);
                        }
                        Decision::Hit | Decision::Double => {
                            self.table.player_hit(i);
                            if self.table.player_hand(i).busted() {
                                outcomes[i] = self.table.get_outcome(i);
                            }
                        }
                        Decision::Stand | Decision::Split => break,
                    }
                }
            }

            processed.extend(bets.into_iter().map(|(_, b)| b));

            if split_hands.is_empty() {
                break;
            }

            bets = split_hands;
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

        self.process_outcomes(processed, &outcomes);
        outcomes
    }

    /// Simulate the strategy for a number of iterations.
    /// Returns the bankroll state at each iter. `vec[t][i]` is the bankroll state of strategy `i` at iteration `t`.
    pub fn simulate(&mut self, iter_max: usize) -> Vec<Vec<usize>> {
        let mut hist = Vec::new();
        hist.push(self.bankrolls.iter().map(|b| b.units()).collect::<Vec<_>>());

        for _ in 0..iter_max {
            let mut bets = Vec::new();
            for (i, strat) in self.strategies.iter().enumerate() {
                for bet in strat.bet() {
                    if self.bankrolls[i].units() >= bet.units() {
                        self.bankrolls[i] -= bet.clone();
                        bets.push((i, bet));
                    }
                }
            }

            if bets.is_empty() {
                break;
            }

            self.run_hand(bets);
            hist.push(self.bankrolls.iter().map(|b| b.units()).collect::<Vec<_>>());
        }

        hist
    }
}
