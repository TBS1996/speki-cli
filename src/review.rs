use crate::{
    add_cards::add_card,
    print_card_info,
    utils::{
        clear_terminal, get_input, notify, select_from_all_cards, select_from_all_class_cards,
        select_from_attributes, select_from_cards,
    },
};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use rand::prelude::*;
use speki_core::{
    attribute::Attribute,
    card::{AnyType, AttributeCard, ClassCard},
    common::CardId,
    reviews::Recall,
    Card,
};
use std::{ops::ControlFlow, str::FromStr};

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
    /// Turn card into an instance of a new class
    NewClass,
    /// Turn card into an instance of an old class
    OldClass,
    NewAttribute,
    OldAttribute,
    FillAttribute,
    SetBackRef,
    /// Turn card into a class
    IntoClass,
    /// Set the parent class of current class
    ParentClass,
    NewCard,
}

#[derive(Clone)]
enum ReviewAction {
    Grade(Recall),
    Help,
    Skip,
}

impl FromStr for CardAction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.trim() {
            "y" => Self::OldDependency,
            "Y" => Self::NewDependency,
            "t" => Self::OldDependent,
            "T" => Self::NewDependent,
            "c" => Self::OldClass,
            "C" => Self::NewClass,
            "i" => Self::IntoClass,
            "p" => Self::ParentClass,
            "a" => Self::OldAttribute,
            "A" => Self::NewAttribute,
            "fa" => Self::FillAttribute,
            "ref" => Self::SetBackRef,
            "n" => Self::NewCard,
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

const DEFAULT_FILTER: &'static str =
    "recall < 0.85 & finished == true & suspended == false & resolved == true & minrecrecall > 0.85 & minrecstab > 50 & lastreview > 0.5 & lapses < 2";

pub fn review_new() {
    let filter = DEFAULT_FILTER.to_string();
    let mut cards = speki_core::Card::load_pending(Some(filter));
    cards.shuffle(&mut thread_rng());

    review(cards);
}

pub fn review_old() {
    let filter = DEFAULT_FILTER.to_string();
    let mut cards = speki_core::Card::load_non_pending(Some(filter));
    cards.shuffle(&mut thread_rng());

    review(cards);
}

fn handle_review_action(card: CardId, action: ReviewAction) -> ControlFlow<()> {
    let card = Card::from_id(card).unwrap();
    match action {
        ReviewAction::Grade(grade) => {
            speki_core::review(card.id(), grade);
            ControlFlow::Break(())
        }
        ReviewAction::Skip => ControlFlow::Break(()),
        ReviewAction::Help => {
            notify(format!("{}", review_help()));
            ControlFlow::Continue(())
        }
    }
}

fn handle_action(card: CardId, action: CardAction) -> ControlFlow<()> {
    let card = Card::from_id(card).unwrap();

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
        CardAction::OldClass => {
            if let Some(concept) = select_from_all_class_cards() {
                speki_core::set_class(card.id(), concept).unwrap();
            }
        }
        CardAction::NewClass => {
            let class: String = Input::new()
                .with_prompt("concept name")
                .allow_empty(true)
                .interact_text()
                .expect("Failed to read input");

            if !class.is_empty() {
                let class = ClassCard {
                    name: class,
                    back: "".to_string().into(),
                    parent_class: None,
                };

                let id = speki_core::Card::new_class(class, card.category()).id();
                speki_core::set_class(card.id(), id).unwrap();
            }
        }

        CardAction::FillAttribute => {
            if let AnyType::Instance(ty) = card.card_type() {
                if Attribute::load_from_concept(ty.class).is_empty() {
                    notify("no attributes for concept. try creating one");
                    return ControlFlow::Continue(());
                }

                if let Some(attribute) = select_from_attributes(ty.class) {
                    let attr = Attribute::load(attribute).unwrap();
                    let txt = attr.name(card.id());

                    let back: String = Input::new()
                        .with_prompt(txt)
                        .allow_empty(true)
                        .interact_text()
                        .expect("Failed to read input");

                    if back.is_empty() {
                        return ControlFlow::Continue(());
                    }

                    let attr = AttributeCard {
                        attribute,
                        back: back.into(),
                        instance: card.id(),
                    };

                    Card::<AttributeCard>::new(attr, card.category());
                }
            }
        }

        CardAction::OldAttribute => {
            let mut dependencies: Vec<CardId> = card.dependency_ids().iter().copied().collect();
            dependencies.retain(|id| Card::from_id(*id).unwrap().is_instance());

            let dependency = if dependencies.len() == 1 {
                Card::from_id(dependencies[0]).unwrap()
            } else if dependencies.is_empty() {
                notify("must have a concept as a dependency");
                return ControlFlow::Continue(());
            } else {
                if let Some(card) = select_from_cards(dependencies) {
                    Card::from_id(card).unwrap()
                } else {
                    return ControlFlow::Continue(());
                }
            };

            let AnyType::Instance(ty) = dependency.card_type() else {
                notify("dependency must be a concept");
                return ControlFlow::Continue(());
            };

            if Attribute::load_from_concept(ty.class).is_empty() {
                notify("no attributes found for concept. try creating one");
                return ControlFlow::Continue(());
            }

            if let Some(attribute) = select_from_attributes(ty.class) {
                let attribute = AttributeCard {
                    attribute,
                    back: card.back_side().unwrap().to_owned(),
                    instance: dependency.id(),
                };

                Card::from_id(card.id()).unwrap().into_attribute(attribute);
            }
        }
        CardAction::NewAttribute => {
            if let AnyType::Instance(ty) = card.card_type() {
                let pattern: String = Input::new()
                    .with_prompt("attribute pattern")
                    .allow_empty(true)
                    .interact_text()
                    .expect("Failed to read input");
                if pattern.is_empty() {
                    notify("no pattern created");
                }

                Attribute::create(pattern, ty.class);
                notify("new pattern created");
            } else {
                notify("current card must be a concept");
            }
        }

        CardAction::IntoClass => {
            let front = card.print();
            let back = card.back_side().map(ToOwned::to_owned).unwrap_or_default();
            let class = ClassCard {
                name: front,
                back,
                parent_class: None,
            };

            card.into_class(class);
        }

        CardAction::ParentClass => {
            if let AnyType::Class(class) = card.card_type() {
                if let Some(parent_class) = select_from_all_class_cards() {
                    if parent_class != card.id() {
                        let mut class = class.clone();
                        class.parent_class = Some(parent_class);
                        card.into_class(class);
                    }
                }
            } else {
                notify("parent class can only be set for class");
            }
        }

        CardAction::SetBackRef => {
            if let Some(reff) = select_from_all_cards() {
                Card::from_id(card.id()).unwrap().set_ref(reff);
            }
        }
        CardAction::Edit => speki_core::edit(card.id()),
        CardAction::Delete => {
            speki_core::delete(card.id());
            return ControlFlow::Break(());
        }

        CardAction::NewCard => {
            let _ = add_card(card.category());
        }
    }

    ControlFlow::Continue(())
}

pub fn view_card(card: CardId, review_mode: bool) -> ControlFlow<()> {
    let mut show_backside = !review_mode;

    loop {
        if print_card(card, show_backside).is_break() {
            return ControlFlow::Continue(());
        }

        show_backside = true;

        let txt: String = get_input("");

        if let Ok(action) = txt.parse::<ReviewAction>() {
            if review_mode {
                match handle_review_action(card, action) {
                    ControlFlow::Continue(_) => continue,
                    ControlFlow::Break(_) => return ControlFlow::Continue(()),
                }
            }
        }

        if let Ok(action) = txt.parse::<CardAction>() {
            match handle_action(card, action) {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(_) => return ControlFlow::Continue(()),
            }
        } else {
            if txt.contains("exit") {
                return ControlFlow::Break(());
            }

            if txt.contains("find") {
                if let Some(card) = select_from_all_cards() {
                    view_card(card, false);
                }

                continue;
            }

            clear_terminal();

            Select::with_theme(&ColorfulTheme::default())
                .with_prompt("write 'help' to see list of possible action")
                .items(&["back to card"])
                .default(0)
                .interact()
                .expect("Failed to make selection");
        };
    }
}

fn print_card(card: CardId, show_backside: bool) -> ControlFlow<()> {
    clear_terminal();
    let card = speki_core::card_from_id(card);
    let (front, back) = {
        if let AnyType::Instance(ty) = card.card_type() {
            let front = format!("which concept: {}", card.print());
            let back = Card::from_id(ty.class).unwrap().print();
            (front, back)
        } else {
            (
                card.print(),
                card.back_side()
                    .map(|bs| bs.to_string())
                    .unwrap_or_default(),
            )
        }
    };

    let opts = ["reveal answer"];

    println!(
        "recall: {:.1}%, stability: {:.2} days, card_type: {}",
        (card.recall_rate().unwrap_or_default() * 100.),
        card.maturity(),
        card.card_type().type_name()
    );
    println!();
    println!("{}", &front);
    if !show_backside {
        println!();
        match Select::with_theme(&ColorfulTheme::default())
            .with_prompt("")
            .items(&opts)
            .default(0)
            .interact()
            .expect("Failed to make selection")
        {
            0 => {
                clear_terminal();
                println!(
                    "recall: {:.1}%, stability: {:.2} days, card_type: {}",
                    (card.recall_rate().unwrap_or_default() * 100.),
                    card.maturity(),
                    card.card_type().type_name()
                );
                println!();
                println!("{}", &front);
                println!();
                println!("-------------------------------------------------");
                println!();
            }
            _ => return ControlFlow::Break(()),
        }
    }

    println!("{}", &back);
    println!();
    print_card_info(card.id());
    ControlFlow::Continue(())
}

pub fn review(cards: Vec<CardId>) {
    if cards.is_empty() {
        clear_terminal();
        notify("nothing to review!");
        return;
    } else {
        clear_terminal();
        notify(format!("reviewing {} cards", cards.len()));
    }

    for card in cards {
        if view_card(card, true).is_break() {
            return;
        }
    }
}
