use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::{categories::Category, common::Id, SavedCard};

#[allow(dead_code)]
pub fn notify(msg: impl Into<String>) {
    clear_terminal();
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(msg.into())
        .items(&["continue"])
        .default(0)
        .interact()
        .unwrap();
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

pub fn get_input(prompt: &str) -> String {
    get_input_opt(prompt).unwrap_or_default()
}

pub fn choose_folder() -> Category {
    let cats: Vec<String> = Category::load_all(None)
        .iter()
        .map(|cat| format!("{}", cat.print_it_with_depth()))
        .collect();

    if cats.len() < 2 {
        return Category::default();
    }

    Category::load_all(None).remove(select_item(&cats))
}

/*

diff reasons we can't sync:

1. not signed in
2. no repo
3. repo, but remote not set
4. remote set, but no access



*/
