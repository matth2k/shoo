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
    /// Blackjack payout
    pub blackjack_payout: (usize, usize),
    /// Offers insurance with 2:1 payout
    pub offers_insurance: bool,
    /// The max number of hands a player can have
    pub max_no_hands: usize,
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
    /// Max level of splitting
    pub max_split_level: usize,
    /// Allow aces to be split again
    pub resplit_aces: bool,
    /// Allow a double after a split
    pub double_after_split: bool,
    /// Split aces only can draw one card
    pub split_aces_one_card: bool,
}

impl Rules {
    /// Returns the rules used in Pennsylvania
    pub fn pennsylvania(
        num_decks: usize,
        penetration: f32,
        max_split_level: usize,
        max_no_hands: usize,
    ) -> Self {
        Self {
            num_decks,
            s17: true,
            blackjack_payout: (3, 2),
            offers_insurance: true,
            max_no_hands,
            penetration,
            random_penetration: false,
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
                Rank::Jack,
                Rank::Queen,
                Rank::King,
            ],
            doubleable: vec![
                2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
            ],
            max_split_level,
            resplit_aces: false,
            double_after_split: true,
            split_aces_one_card: true,
            late_surrender: true,
        }
    }
}

impl Default for Rules {
    fn default() -> Self {
        Rules::pennsylvania(6, 0.7, 4, 4)
    }
}

/// Decisions needed to play blackjack
pub trait Strategy {
    /// Place a series of bets/hands in the current context
    fn bet(&self) -> Vec<Bet>;

    /// Take an insurance bet
    fn insure(&self) -> Option<Bet>;

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
        let (n, d) = self.rules.blackjack_payout;
        for ((strat, bet), &outcome) in bets.into_iter().zip(outcomes) {
            match outcome {
                Outcome::Win => self.bankrolls[strat] += bet * 2,
                Outcome::Lose => {}
                Outcome::Blackjack => self.bankrolls[strat] += (bet * (n + d)) / d,
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

        let mut insurance_bets: Vec<(usize, Bet)> = Vec::new();
        if self.table.upcard().is_some_and(|c| c.is_ace()) && self.rules.offers_insurance {
            for (i, strat) in self.strategies.iter().enumerate() {
                if let Some(bet) = strat.insure()
                    && self.bankrolls[i] >= bet
                {
                    self.bankrolls[i] -= bet.clone();
                    insurance_bets.push((i, bet));
                }
            }
        }

        if self.table.peek() {
            for (strat, bet) in insurance_bets {
                self.bankrolls[strat] += bet * 3;
            }

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
                        _ if hand.split_level() > 0
                            && hand.first().is_some_and(|c| c.is_ace())
                            && self.rules.split_aces_one_card =>
                        {
                            break 'deciding;
                        }
                        Decision::Double
                            if hand.len() == 2
                                && self.bankrolls[*strat] >= *bet
                                && let Some(v) = hand.value()
                                && self.rules.doubleable.contains(&v)
                                && (self.rules.double_after_split || hand.split_level() == 0) =>
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
                                && hand.split_level() < self.rules.max_split_level
                                && !(front.is_ace()
                                    && !self.rules.resplit_aces
                                    && hand.split_level() > 0)
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
            .any(|&outcome| !matches!(outcome, Outcome::Lose | Outcome::Forfeit))
        {
            while let Some(v) = self.table.dealer_value()
                && (v < 17 || (v == 17 && self.table.dealer_hand().is_soft() && !self.rules.s17))
            {
                self.table.dealer_hit();
            }
        }

        for (spot, outcome) in spots
            .iter()
            .zip(outcomes.iter_mut())
            .filter(|(_, o)| !matches!(o, Outcome::Lose | Outcome::Forfeit))
        {
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
                for bet in strat.bet().into_iter().take(self.rules.max_no_hands) {
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
