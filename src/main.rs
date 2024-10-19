use add_cards::add_cards_menu;
use clap::Parser;
use collections::col_stuff;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use incread::inc_path;
use opener::open;
use review::review_menu;
use speki_core::{
    categories::Category,
    common::Id,
    github::{poll_for_token, request_device_code, LoginInfo},
    paths::{config_dir, get_cards_path, get_review_path},
    SavedCard,
};
use utils::{clear_terminal, notify};

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

async fn menu() {
    let mut login = LoginInfo::load();

    //new_repo_col(&login.clone().unwrap(), "repo3", true);

    loop {
        utils::clear_terminal();
        if let Some(info) = &login {
            println!("signed in as {}", info.login)
        };

        let sign = if login.is_some() {
            "sign out"
        } else {
            "sign in"
        };

        let items = vec![
            "Review cards",
            "Add cards",
            "Manage collections",
            "Inspect files",
            "sync",
            sign,
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .items(&items)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => review_menu(),
            1 => add_cards_menu().await,
            2 => col_stuff(),
            3 => inspect_files(),
            4 => {
                if let Some(login) = LoginInfo::load() {
                    speki_core::github::sync(&login);
                }
            }
            5 => match login.take() {
                Some(login) => login.delete_login(),
                None => login = Some(authenticate()),
            },
            _ => panic!(),
        }
    }
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

    println!("{}", style("dependendents").bold());
    for id in speki_core::get_cached_dependents(id) {
        println!("{}", SavedCard::from_id(&id).unwrap().front_text());
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

pub fn authenticate() -> LoginInfo {
    clear_terminal();
    let res = request_device_code().unwrap();
    open(&res.verification_uri).unwrap();
    clear_terminal();
    notify(format!("enter code in browser: {}", &res.user_code));
    let token = poll_for_token(&res.device_code, res.interval);
    token
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.add.is_some() {
        let s = cli.add.unwrap();
        let category = Category::default();

        if let Some((front, back)) = s.split_once(";") {
            speki_core::add_card(front.to_string(), back.to_string(), &category);
        } else {
            speki_core::add_unfinished(s, &category);
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
        menu().await;
    }
}
