use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::{categories::Category, common::Id};

use crate::utils::{choose_folder, clear_terminal};

pub fn add_cards() {
    let category = choose_folder();

    loop {
        clear_terminal();
        if add_card(&category).is_none() {
            break;
        }
    }
}

pub fn add_card(category: &Category) -> Option<Id> {
    let s = style("front").bold();
    let front: String = Input::new()
        .with_prompt(s.to_string())
        .allow_empty(true)
        .interact_text()
        .expect("Failed to read input");

    if front.trim().is_empty() {
        return None;
    }

    let s = style("back").bold();
    let back: String = Input::new()
        .with_prompt(s.to_string())
        .allow_empty(true)
        .interact_text()
        .expect("Failed to read input");

    if back.trim().is_empty() {
        let opts = ["add as unfinished", "exit"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("")
            .items(&opts)
            .default(0)
            .interact()
            .unwrap();

        match selection {
            0 => speki_core::add_unfinished(front, category),
            1 => return None,
            _ => panic!(),
        }
    } else {
        speki_core::add_card(front, back, category)
    }
    .into()
}
