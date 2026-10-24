use std::io::Write;

use crossterm::{
    event::{Event, KeyCode, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use shoo::table::{Outcome, Table};

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;

    let num_decks = 1;
    let max_penetration = 0.40;

    println!("Num decks: {num_decks} Max Pen: {max_penetration} \n\r\n");
    let mut table = Table::new(num_decks, 1, max_penetration);
    let mut last_outcome = Some(Outcome::Push);
    loop {
        if let Event::Key(key_event) = read()? {
            // Only trigger on press (not release)
            if key_event.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }
            match key_event.code {
                KeyCode::Right => {
                    match last_outcome {
                        Some(_) => {
                            if table.deal_all() {
                                println!("\rShoe reshuffled!            ");
                            }
                            if table.peek() {
                                print!("\rDealer has blackjack!            ");
                                std::io::stdout().flush()?;
                                last_outcome = Some(table.get_outcome(0));
                                table.flip_hole();
                                table.clear_hands();
                                continue;
                            }

                            if table.player_hand(0).blackjack() {
                                print!("\rBlackjack!            ");
                                std::io::stdout().flush()?;
                                last_outcome = Some(table.get_outcome(0));
                                table.flip_hole();
                                table.clear_hands();
                                continue;
                            }

                            print!("\r{table}");
                            std::io::stdout().flush()?;
                            last_outcome = None;
                        }
                        _ => {
                            // A hit
                            table.player_hit(0);

                            if table.player_hand(0).busted() {
                                print!("\rBust!            ");
                                print!("\r{table}");
                                std::io::stdout().flush()?;
                                last_outcome = Some(table.get_outcome(0));
                                table.flip_hole();
                                table.clear_hands();
                                continue;
                            }
                            print!("\r{table}");
                            std::io::stdout().flush()?;
                        }
                    }
                }
                KeyCode::Down => {
                    if last_outcome.is_some() {
                        continue;
                    }

                    table.flip_hole();
                    while table.dealer_value().is_some_and(|v| v < 17) {
                        table.dealer_hit();
                        print!("\r{table}");
                        std::io::stdout().flush()?;
                        // Sleep for one second
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }

                    let outcome = table.get_outcome(0);
                    match outcome {
                        Outcome::Blackjack | Outcome::Win => {
                            print!("\rYou win!                           ");
                        }
                        Outcome::Push => {
                            print!("\rPush!                              ");
                        }
                        Outcome::Lose => {
                            print!("\rYou lose!                          ");
                        }
                    };
                    std::io::stdout().flush()?;
                    last_outcome = Some(outcome);
                    table.clear_hands();
                }
                KeyCode::Char('c')
                    if key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL) =>
                {
                    break;
                }
                KeyCode::Esc => {
                    break;
                }
                _ => (),
            }
        }
    }

    disable_raw_mode()
}
