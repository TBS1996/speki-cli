use crate::{print_dependencies, read, utils::clear_terminal, utils::select_from_all_cards};
use dialoguer::{theme::ColorfulTheme, Select};
use speki_core::reviews::Recall;
use std::str::FromStr;

fn review_help() -> &'static str {
    r#"
    
possible commands:

1 =>        failed to recall backside, where the backside info seems new to you
2 =>        failed ot recall backside but the information was familiar to you when reading it
3 =>        successfully recalled backside after some thinking
4 =>        successfully recalled backside without hesitation
skip | s => skip card
y =>        add new dependency, from cards in your collections
t =>        add new dependent, from cards in your collections
Y =>        add new dependency by creating a new card
T =>        add new dependent, by creating a new card
edit =>     open the card in vim (must be installed)
delete =>   delete the card
exit =>     back to main menu
help | ? => open this help message
    "#
}

#[derive(Clone)]
enum ReviewAction {
    Grade(Recall),
    NewDependency,
    OldDependency,
    NewDependent,
    OldDependent,
    Edit,
    Delete,
    Exit,
    Help,
    Skip,
}

impl ReviewAction {
    fn next_card(&self) -> bool {
        match self {
            ReviewAction::Grade(_) => true,
            ReviewAction::Skip => true,
            ReviewAction::Delete => true,
            ReviewAction::NewDependency => false,
            ReviewAction::OldDependency => false,
            ReviewAction::NewDependent => false,
            ReviewAction::OldDependent => false,
            ReviewAction::Edit => false,
            ReviewAction::Exit => false,
            ReviewAction::Help => false,
        }
    }
}

impl FromStr for ReviewAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "1" => Self::Grade(Recall::None),
            "2" => Self::Grade(Recall::Late),
            "3" => Self::Grade(Recall::Some),
            "4" => Self::Grade(Recall::Perfect),
            "y" => Self::OldDependency,
            "Y" => Self::NewDependency,
            "t" => Self::OldDependent,
            "T" => Self::NewDependent,
            "edit" => Self::Edit,
            "delete" => Self::Delete,
            "exit" => Self::Exit,
            "help" | "?" => Self::Help,
            "skip" | "s" => Self::Skip,
            _ => return Err(()),
        })
    }
}

pub fn review() {
    let filter =
        "recall < 0.9 & finished == true & suspended == false & resolved == true & minrecrecall > 0.9 & minrecstab > 10 & lastreview > 0.5".to_string();

    /*
    let filter: String = Input::new()
        .with_prompt("filter")
        .allow_empty(true)
        .with_initial_text(filter)
        .interact_text()
        .expect("Failed to read input");

    if filter.is_empty() {
        return;
    }
    */

    let cards = speki_core::cards_filtered(filter);
    if cards.is_empty() {
        clear_terminal();
        println!("nothing to review!");
        read();
        return;
    } else {
        clear_terminal();
        println!("reviewing {} cards", cards.len());
        read();
    }

    for card in cards {
        let mut flag = false;

        loop {
            clear_terminal();
            let card = speki_core::card_from_id(card);

            let opts = ["reveal answer"];

            println!(
                "recall: {:.1}% stability: {:.2} days",
                (card.recall_rate().unwrap() * 100.),
                card.maturity()
            );
            println!();
            println!("{}", card.front_text());
            if !flag {
                println!();
                match Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("")
                    .items(&opts)
                    .default(0)
                    .interact()
                    .expect("Failed to make selection")
                {
                    0 => {
                        flag = true;
                        clear_terminal();
                        println!(
                            "recall: {:.1}% stability: {:.2} days",
                            (card.recall_rate().unwrap() * 100.),
                            card.maturity()
                        );
                        println!();
                        println!("{}", card.front_text());
                        println!();
                        println!("-------------------------------------------------");
                        println!();
                    }
                    _ => return,
                }
            }

            println!("{}", card.back_text());
            if !card.dependency_ids().is_empty() {
                println!();
                print_dependencies(card.id());
            }

            let txt: String = read();

            let Ok(action) = txt.parse::<ReviewAction>() else {
                clear_terminal();

                Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("write 'help' to see list of possible action")
                    .items(&["back to card"])
                    .default(0)
                    .interact()
                    .expect("Failed to make selection");

                continue;
            };

            match action.clone() {
                ReviewAction::Grade(grade) => speki_core::review(card.id(), grade),
                ReviewAction::NewDependency => {
                    println!("add dependency");
                    let s: String = read();
                    let id = if let Some((front, back)) = s.split_once(";") {
                        speki_core::add_card(front.to_string(), back.to_string(), card.category())
                    } else {
                        speki_core::add_unfinished(s, card.category())
                    };

                    speki_core::set_dependency(card.id(), id);
                }
                ReviewAction::OldDependency => {
                    if let Some(dep) = select_from_all_cards() {
                        speki_core::set_dependency(card.id(), dep);
                    }
                }
                ReviewAction::NewDependent => {
                    println!("add dependent");
                    let s: String = read();
                    let id = if let Some((front, back)) = s.split_once(";") {
                        speki_core::add_card(front.to_string(), back.to_string(), card.category())
                    } else {
                        speki_core::add_unfinished(s, card.category())
                    };

                    speki_core::set_dependency(id, card.id());
                }
                ReviewAction::OldDependent => {
                    if let Some(dep) = select_from_all_cards() {
                        speki_core::set_dependency(dep, card.id());
                    }
                }
                ReviewAction::Edit => speki_core::edit(card.id()),
                ReviewAction::Delete => speki_core::delete(card.id()),
                ReviewAction::Exit => return,
                ReviewAction::Skip => break,
                ReviewAction::Help => {
                    clear_terminal();
                    println!("{}", review_help());
                    read();
                }
            }

            if action.next_card() {
                break;
            } else {
                continue;
            };
        }
    }
}
