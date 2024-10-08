use clap::Parser;
use collections::col_stuff;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use incread::{inc_path, textstuff};
use speki_core::{
    common::Id,
    paths::{config_dir, get_cards_path, get_review_path},
    SavedCard,
};
use text_io::read;

mod add_cards;
mod collections;
mod incread;
mod review;
mod unfinished;
mod utils;

fn inspect_files() {
    let items = vec![
        "Inspect config",
        "Inspect cards",
        "Inspect reviews",
        "Inspect texts",
        "go back",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .items(&items)
        .default(0)
        .interact()
        .unwrap();

    match selection {
        0 => opener::open(config_dir()).unwrap(),
        1 => opener::open(get_cards_path()).unwrap(),
        2 => opener::open(get_review_path()).unwrap(),
        3 => opener::open(inc_path()).unwrap(),
        4 => {}
        _ => panic!(),
    }
}

fn menu() {
    loop {
        utils::clear_terminal();
        let items = vec![
            "Review",
            "Add cards",
            "Unfinished",
            "Incremental reading",
            "Inspect collections",
            "Inspect files",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("")
            .items(&items)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => crate::review::review(),
            1 => crate::add_cards::add_cards(),
            2 => crate::unfinished::unfinished(),
            3 => textstuff(),
            4 => col_stuff(),
            5 => inspect_files(),
            _ => panic!(),
        }
    }
}

fn read() -> String {
    read!("{}\n")
}

fn print_dependencies(id: Id) {
    let card = SavedCard::from_id(&id).unwrap();
    let dependencies = card.dependency_ids();
    if dependencies.is_empty() {
        return;
    }

    println!("{}", style("dependencies").bold());
    for id in dependencies {
        println!("{}", SavedCard::from_id(id).unwrap().front_text());
    }
    println!();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    add: Option<String>,
    #[arg(short, long)]
    filter: Option<String>,
    #[arg(short, long)]
    list: bool,
    #[arg(short, long)]
    graph: bool,
    #[arg(short, long)]
    prune: bool,
    #[arg(long)]
    debug: bool,
    #[arg(long)]
    recall: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if cli.add.is_some() {
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
    } else if cli.debug {
        //speki_core::fetch_repos();
        // speki_core::categories::Category::load_all();
    } else if cli.recall.is_some() {
        let id = cli.recall.unwrap();
        let id: uuid::Uuid = id.parse().unwrap();
        let x = speki_core::SavedCard::from_id(&id).unwrap().recall_rate();
        dbg!(x);
    } else {
        menu();
    }
}
