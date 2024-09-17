use clap::Parser;
use skim::prelude::*;
use speki_core::{common::Id, Grade, SavedCard};
use std::str::FromStr;
use std::{collections::HashMap, io::Cursor};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use strum_macros::{Display, EnumString};
use text_io::read;

fn read() -> String {
    read!("{}\n")
}

trait CardSelector: IntoEnumIterator + ToString + FromStr + std::fmt::Debug {
    fn select() -> Option<Self> {
        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .multi(false)
            .reverse(true)
            .build()
            .unwrap();

        let vec: Vec<String> = Self::iter().map(|x| x.to_string()).collect();
        let choices: String = vec.join("\n");
        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(choices));

        let selected_item = Skim::run_with(&options, Some(items))?;
        if selected_item.is_abort {
            return None;
        }

        let selected_item: String = selected_item.selected_items.first()?.output().to_string();

        Some(Self::from_str(&selected_item).ok()?)
    }
}

fn select_card(ids: Vec<Id>) -> Option<Id> {
    clear_terminal();
    let options = SkimOptionsBuilder::default()
        .height(Some("50%"))
        .multi(true)
        .build()
        .unwrap();

    let mut txt_to_id = HashMap::new();
    let mut input = vec![];

    for card in ids {
        let card = SavedCard::from_id(&card).unwrap();
        txt_to_id.insert(card.front_text().to_string(), card.id());
        input.push(card.front_text().to_string());
    }

    let input_str = input.join("\n");

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input_str));

    let selected_item = Skim::run_with(&options, Some(items))?;
    if selected_item.is_abort {
        return None;
    }

    let selected_item: String = selected_item
        .selected_items
        .first()?
        .output()
        .parse()
        .unwrap();

    Some(txt_to_id[&selected_item])
}

#[derive(EnumIter, Debug, EnumString, Display)]
enum Menu {
    Review,
    View,
}

impl CardSelector for Menu {}

fn print_dependencies(id: Id) {
    let card = SavedCard::from_id(&id).unwrap();
    let dependencies = card.dependency_ids();
    if dependencies.is_empty() {
        return;
    }

    print!("dependencies:  ");
    for id in dependencies {
        print!("{}, ", SavedCard::from_id(id).unwrap().front_text());
    }
    println!();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    add: Option<String>,
    #[arg(short, long)]
    review: bool,
    #[arg(short, long)]
    filter: Option<String>,
    #[arg(short, long)]
    list: bool,
    #[arg(short, long)]
    graph: bool,
    #[arg(short, long)]
    prune: bool,
    #[arg(long)]
    addmany: bool,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    unfinished: bool,
}

fn add_card() -> Id {
    let s = read();
    if let Some((front, back)) = s.split_once(";") {
        speki_core::add_card(front.to_string(), back.to_string())
    } else {
        speki_core::add_unfinished(s)
    }
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[H");
}

fn main() {
    for card in SavedCard::load_all_cards() {
        card.save_reviews();
    }

    let cli = Cli::parse();
    let default_review_filter =
        "recall < 0.9 & finished == true & suspended == false & resolved == true".to_string();

    if cli.review {
        let filter = cli.filter.unwrap_or(default_review_filter);
        let cards = speki_core::cards_filtered(filter);

        for card in cards {
            let mut flag = false;

            loop {
                clear_terminal();
                let card = speki_core::card_from_id(card);
                println!("{}", card.front_text());
                if !flag {
                    read();
                    flag = true;
                }
                print_dependencies(card.id());
                println!("{}", card.back_text());
                let txt: String = read();
                match txt.as_str() {
                    "1" => speki_core::review(card.id(), Grade::None),
                    "2" => speki_core::review(card.id(), Grade::Late),
                    "3" => speki_core::review(card.id(), Grade::Some),
                    "4" => speki_core::review(card.id(), Grade::Perfect),
                    "delete" => {
                        speki_core::delete(card.id());
                    }
                    "edit" => {
                        speki_core::edit(card.id());
                        continue;
                    }
                    "t" => {
                        if let Some(dep) = select_from_all_cards() {
                            speki_core::set_dependency(dep, card.id());
                        }
                        continue;
                    }
                    "y" => {
                        if let Some(dep) = select_from_all_cards() {
                            speki_core::set_dependency(card.id(), dep);
                        }
                        continue;
                    }
                    "T" => {
                        println!("add dependent");
                        let s: String = read();
                        let id = if let Some((front, back)) = s.split_once(";") {
                            speki_core::add_card(front.to_string(), back.to_string())
                        } else {
                            speki_core::add_unfinished(s)
                        };

                        speki_core::set_dependency(id, card.id());
                        continue;
                    }
                    "Y" => {
                        println!("add dependency");
                        let s: String = read();
                        let id = if let Some((front, back)) = s.split_once(";") {
                            speki_core::add_card(front.to_string(), back.to_string())
                        } else {
                            speki_core::add_unfinished(s)
                        };

                        speki_core::set_dependency(card.id(), id);
                        continue;
                    }
                    "?" => {
                        dbg!(&card);
                        read();
                        continue;
                    }
                    _ => {}
                }

                break;
            }
        }
    } else if cli.add.is_some() {
        let s = cli.add.unwrap();

        if let Some((front, back)) = s.split_once(";") {
            speki_core::add_card(front.to_string(), back.to_string());
        } else {
            speki_core::add_unfinished(s);
        }
    } else if cli.list {
        dbg!(speki_core::load_cards());
    } else if cli.graph {
        println!("{}", speki_core::as_graph());
    } else if cli.prune {
        speki_core::prune_dependencies();
    } else if cli.addmany {
        loop {
            add_card();
        }
    } else if cli.debug {
        let id = select_from_all_cards();
        dbg!(SavedCard::from_id(&id.unwrap()));
    } else if cli.unfinished {
        let filter = "finished == false".to_string();
        let cards = speki_core::cards_filtered(filter);

        for id in cards {
            let card = SavedCard::from_id(&id).unwrap();
            clear_terminal();
            println!("{}", card.front_text());
            let answer = read();

            if answer.is_empty() {
                continue;
            } else {
                speki_core::set_back_text(id, answer);
                speki_core::set_finished(id, true);
            }
        }
    }
}

fn select_from_all_cards() -> Option<Id> {
    let mut ids = vec![];

    for card in SavedCard::load_all_cards() {
        ids.push(card.id());
    }

    select_card(ids)
}
