#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub)]
/*!

`shoo`

Library for blackjack games and trainers.

*/
mod bet;
mod card;
mod hand;
mod shoe;
pub mod strategy;
pub mod table;

pub use bet::*;
pub use card::*;
pub use hand::*;
pub use shoe::*;
