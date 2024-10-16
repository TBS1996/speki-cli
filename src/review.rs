use crate::{
    add_cards::add_card,
    print_dependencies,
    utils::{clear_terminal, get_input, notify, select_from_all_cards},
};
use dialoguer::{theme::ColorfulTheme, Select};
use speki_core::{common::Id, reviews::Recall};
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
enum CardAction {
    NewDependency,
    OldDependency,
    NewDependent,
    OldDependent,
    Edit,
    Delete,
}

impl CardAction {
    fn next_card(&self) -> bool {
        match self {
            CardAction::NewDependency => false,
            CardAction::OldDependency => false,
            CardAction::NewDependent => false,
            CardAction::OldDependent => false,
            CardAction::Delete => true,
            CardAction::Edit => false,
        }
    }
}

#[derive(Clone)]
enum ReviewAction {
    Grade(Recall),
    Help,
    Skip,
}

impl ReviewAction {
    fn next_card(&self) -> bool {
        match self {
            ReviewAction::Grade(_) => true,
            ReviewAction::Skip => true,
            ReviewAction::Help => false,
        }
    }
}

impl FromStr for CardAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "y" => Self::OldDependency,
            "Y" => Self::NewDependency,
            "t" => Self::OldDependent,
            "T" => Self::NewDependent,
            "edit" => Self::Edit,
            "delete" => Self::Delete,
            _ => return Err(()),
        })
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
            "help" | "?" => Self::Help,
            "skip" | "s" => Self::Skip,
            _ => return Err(()),
        })
    }
}

pub fn review_menu() {
    let items = vec!["Old cards", "Pending cards", "exit"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    match selection {
        0 => review_old(),
        1 => review_new(),
        2 => return,
        _ => panic!(),
    }
}

pub fn review_new() {
    let filter =
        "recall < 0.9 & finished == true & suspended == false & resolved == true & minrecrecall > 0.9 & minrecstab > 10 & lastreview > 0.5 & lapses < 4".to_string();
    let cards = speki_core::SavedCard::load_pending(Some(filter.to_owned()));

    review(cards);
}

pub fn review_old() {
    let filter =
        "recall < 0.9 & finished == true & suspended == false & resolved == true & minrecrecall > 0.9 & minrecstab > 10 & lastreview > 0.5 & lapses < 2".to_string();
    let cards = speki_core::SavedCard::load_non_pending(Some(filter.to_owned()));

    review(cards);
}

pub fn review(cards: Vec<Id>) {
    if cards.is_empty() {
        clear_terminal();
        notify("nothing to review!");
        return;
    } else {
        clear_terminal();
        notify(format!("reviewing {} cards", cards.len()));
    }

    for card in cards {
        let mut flag = false;

        loop {
            clear_terminal();
            let card = speki_core::card_from_id(card);

            let opts = ["reveal answer"];

            println!(
                "recall: {:.1}% stability: {:.2} days",
                (card.recall_rate().unwrap_or_default() * 100.),
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
                            (card.recall_rate().unwrap_or_default() * 100.),
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

            let txt: String = get_input("");

            if let Ok(action) = txt.parse::<ReviewAction>() {
                match action.clone() {
                    ReviewAction::Grade(grade) => speki_core::review(card.id(), grade),
                    ReviewAction::Skip => break,
                    ReviewAction::Help => {
                        notify(format!("{}", review_help()));
                    }
                }

                if action.next_card() {
                    break;
                } else {
                    continue;
                };
            } else if let Ok(action) = txt.parse::<CardAction>() {
                match action.clone() {
                    CardAction::NewDependency => {
                        println!("add dependency");
                        if let Some(new_card) = add_card(card.category()) {
                            speki_core::set_dependency(card.id(), new_card);
                        }
                    }
                    CardAction::OldDependency => {
                        if let Some(dep) = select_from_all_cards() {
                            speki_core::set_dependency(card.id(), dep);
                        }
                    }
                    CardAction::NewDependent => {
                        println!("add dependent");
                        if let Some(new_card) = add_card(card.category()) {
                            speki_core::set_dependency(new_card, card.id());
                        }
                    }
                    CardAction::OldDependent => {
                        if let Some(dep) = select_from_all_cards() {
                            speki_core::set_dependency(dep, card.id());
                        }
                    }
                    CardAction::Edit => speki_core::edit(card.id()),
                    CardAction::Delete => speki_core::delete(card.id()),
                }
                if action.next_card() {
                    break;
                } else {
                    continue;
                };
            } else {
                clear_terminal();

                Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("write 'help' to see list of possible action")
                    .items(&["back to card"])
                    .default(0)
                    .interact()
                    .expect("Failed to make selection");

                continue;
            };
        }
    }
}
