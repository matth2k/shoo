/*!

  Simulate a blackjack strategy

*/

use crate::table::{Outcome, Table};
use crate::{Bet, Card, Hand, Rank};

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
    /// Surrender
    Surrender,
}

/// Variant of blackjack
#[derive(Debug, Clone, PartialEq)]
pub struct Rules {
    /// The number of decks in the shoe
    pub num_decks: usize,
    /// Stand on all seventeens
    pub s17: bool,
    /// Deck penetration (0.0-0.85)
    pub penetration: f32,
    /// penetration is normally random
    pub random_penetration: bool,
    /// Late surrender is offered
    pub late_surrender: bool,
    /// The splittable cards
    pub splittable: Vec<Rank>,
    /// Hands that can be doubled
    pub doubleable: Vec<u8>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            num_decks: 6,
            s17: true,
            penetration: 0.75,
            random_penetration: true,
            late_surrender: true,
            splittable: vec![
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
            ],
            doubleable: vec![9, 10, 11],
        }
    }
}

/// Decisions needed to play blackjack
pub trait Strategy {
    /// Place a series of bets/hands in the current context
    fn bet(&self) -> Vec<Bet>;

    /// Make a choice based on the hand and upcard
    fn decide(&self, hand: &Hand, upcard: Card) -> Decision;

    /// Process the last outcome of the hand
    fn process(&mut self, hands: &[Hand], dealer: &Hand, outcomes: &[Outcome]);

    /// Hook for when the cards are shuffled. This is useful for card counting strategies.
    fn shoe_shuffled(&mut self);

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
    /// The rules of the game
    rules: Rules,
}

impl Simulator {
    /// Create a new simulator with the given strategy and bankroll
    pub fn new<S: Strategy + 'static>(strategy: S, bankroll: Bet, rules: Rules) -> Self {
        let bankrolls = vec![bankroll];
        let pen_dev = if rules.random_penetration {
            Some(0.1)
        } else {
            None
        };
        let table = Table::new(rules.num_decks, rules.penetration, pen_dev);
        let boxed: Box<dyn Strategy> = Box::new(strategy);
        let strategies = vec![boxed];
        Self {
            strategies,
            bankrolls,
            table,
            rules,
        }
    }

    /// Add an additional strategy to the simulator
    pub fn insert<S: Strategy + 'static>(&mut self, strategy: S, bankroll: Bet) {
        self.bankrolls.push(bankroll);
        let boxed: Box<dyn Strategy> = Box::new(strategy);
        self.strategies.push(boxed);
    }

    /// Update the bankroll and let strategies process the outcomes
    fn process_outcomes(&mut self, spots: &[usize], bets: Vec<(usize, Bet)>, outcomes: &[Outcome]) {
        let hands: Vec<Hand> = spots
            .iter()
            .map(|s| self.table.player_hand(*s).clone())
            .collect();
        for strat in &mut self.strategies {
            strat.process(&hands, self.table.dealer_hand(), outcomes);
        }

        self.update_bankroll(bets, outcomes);
        self.table.clear_hands();
    }

    /// Add or subtract for the bankrolls
    fn update_bankroll(&mut self, bets: Vec<(usize, Bet)>, outcomes: &[Outcome]) {
        for ((strat, bet), &outcome) in bets.into_iter().zip(outcomes) {
            let double = bet.clone() * 2;
            match outcome {
                Outcome::Win => self.bankrolls[strat] += double,
                Outcome::Lose => {}
                Outcome::Blackjack => self.bankrolls[strat] += double + (bet / 2),
                Outcome::Push => self.bankrolls[strat] += bet,
                Outcome::Forfeit => self.bankrolls[strat] += bet / 2,
            }
        }
    }

    /// Run a single iteration with a set of bets. Returns the outcomes of the hands.
    fn run_hand(&mut self, mut bets: Vec<(usize, Bet)>) -> Vec<Outcome> {
        if bets.is_empty() {
            return Vec::new();
        }

        // Register the initial bettingspots
        let mut spots: Vec<usize> = bets
            .iter()
            .map(|_| self.table.get_spot())
            .collect::<Vec<_>>();

        if self.table.deal() {
            for strat in self.strategies.iter_mut() {
                strat.shoe_shuffled();
            }
        }

        if self.table.peek() {
            let outcomes: Vec<_> = spots.iter().map(|&i| self.table.get_outcome(i)).collect();
            self.process_outcomes(&spots, bets, &outcomes);
            return outcomes;
        }

        let upcard = self.table.upcard().unwrap();
        let mut outcomes = vec![Outcome::Push; self.table.num_spots()];
        let mut starting_spot = 0;

        'outer: loop {
            let mut new_bets = Vec::new();
            let mut new_spots: Vec<usize> = Vec::new();

            debug_assert_eq!(spots.len(), outcomes.len());
            debug_assert_eq!(spots.len(), bets.len());

            'inner: for (k, (((strat, bet), outcome), spot)) in bets
                .iter_mut()
                .zip(outcomes.iter_mut())
                .zip(spots.iter())
                .enumerate()
                .skip(starting_spot)
            {
                'deciding: while let hand = self.table.player_hand(*spot)
                    && !hand.busted()
                {
                    match self.strategies[*strat].decide(hand, upcard) {
                        Decision::Double
                            if hand.len() == 2
                                && self.bankrolls[*strat] >= *bet
                                && let Some(v) = hand.value()
                                && self.rules.doubleable.contains(&v) =>
                        {
                            self.bankrolls[*strat] -= bet.clone();
                            *bet = bet.clone() * 2;
                            self.table.player_hit(*spot);
                            if self.table.player_hand(*spot).busted() {
                                *outcome = self.table.get_outcome(*spot);
                            }
                            break 'deciding;
                        }
                        Decision::Surrender if hand.len() == 2 && self.rules.late_surrender => {
                            *outcome = Outcome::Forfeit;
                            break 'deciding;
                        }
                        Decision::Split
                            if self.bankrolls[*strat] >= *bet
                                && let Some(front) = hand.first()
                                && self.rules.splittable.contains(&front.rank())
                                && let Some((_, other)) = self.table.player_split(*spot) =>
                        {
                            self.bankrolls[*strat] -= bet.clone();
                            new_spots.push(other);
                            new_bets.push((*strat, bet.clone()));
                            starting_spot = k;
                            break 'inner;
                        }
                        Decision::Hit | Decision::Double | Decision::Surrender => {
                            self.table.player_hit(*spot);
                            if self.table.player_hand(*spot).busted() {
                                *outcome = self.table.get_outcome(*spot);
                            }
                        }
                        Decision::Stand | Decision::Split => {
                            break 'deciding;
                        }
                    }
                }
            }

            if new_bets.is_empty() {
                break 'outer;
            }

            let insert_index = starting_spot + 1;
            for _ in 0..new_bets.len() {
                outcomes.insert(insert_index, Outcome::Push);
            }
            bets.splice(insert_index..insert_index, new_bets);
            spots.splice(insert_index..insert_index, new_spots);
        }

        self.table.flip_hole();
        if outcomes
            .iter()
            .any(|&outcome| outcome != Outcome::Lose && outcome != Outcome::Forfeit)
        {
            while let Some(v) = self.table.dealer_value()
                && (v < 17 || (v == 17 && self.table.dealer_hand().is_soft() && !self.rules.s17))
            {
                self.table.dealer_hit();
            }
        }

        for (spot, outcome) in spots.iter().zip(outcomes.iter_mut()) {
            *outcome = self.table.get_outcome(*spot);
        }

        self.process_outcomes(&spots, bets, &outcomes);
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
                    if self.bankrolls[i] >= bet {
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

    /// Returns the rules of the game
    pub fn rules(&self) -> &Rules {
        &self.rules
    }
}
