use crate::read;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::{categories::Category, collections::Collection, common::Id, SavedCard};

pub fn _notify(msg: &str) {
    clear_terminal();
    println!("{}", msg);
    read();
}

pub fn select_from_all_cards() -> Option<Id> {
    enumselector::select_item_with_formatter(SavedCard::load_all_cards(), |card: &SavedCard| {
        card.front_text().to_owned()
    })?
    .id()
    .into()
}

pub fn clear_terminal() {
    use std::io::Write;
    print!("\x1B[2J\x1B[H");
    std::io::stdout().flush().unwrap();
}

pub fn get_lines(text: &str, line_width: usize, height: usize, position: usize) -> Vec<String> {
    let mut output = vec![];
    let lines = cli_justify::justify(text, line_width);

    let mut sum = 0;
    for line in lines {
        sum += line.chars().count();
        if sum >= position {
            output.push(line);
        }

        if output.len() >= height {
            return output;
        }
    }

    output
}

pub fn select_item<T: ToString>(items: &[T]) -> usize {
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .items(items)
        .default(0)
        .interact()
        .unwrap()
}

pub fn get_input_opt(prompt: &str) -> Option<String> {
    let s: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()
        .expect("Failed to read input");

    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn _get_input(prompt: &str) -> String {
    get_input_opt(prompt).unwrap_or_default()
}

pub fn choose_folder() -> Option<Category> {
    let mut opts: Vec<String> = Collection::load_all()
        .iter()
        .map(|col| col.name().to_string())
        .collect();
    opts.push("exit".to_string());
    let selection = select_item(&opts);
    if selection == opts.len() {
        return None;
    }

    let col = Collection::load_all().remove(selection);

    let cats: Vec<String> = Category::load_all(&col)
        .iter()
        .map(|cat| format!("{}", cat.print_it_with_depth()))
        .collect();

    let category = Category::load_all(&col).remove(select_item(&cats));

    Some(category)
}
