use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::{common::Id, SavedCard};

use crate::read;

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

pub fn get_lines_slice(chars: &[char], max_lines: usize) -> String {
    let width = console::Term::stdout().size().0;

    let mut line_count = 0;
    let mut current_line_length = 0;
    let mut result = String::new();

    for &c in chars {
        // If we hit a newline, reset the current line length
        if c == '\n' {
            result.push(c);
            line_count += 1;
            current_line_length = 0;
        } else {
            // Add character to the result
            result.push(c);
            current_line_length += 1;

            // If the current line exceeds terminal width, consider it wrapped
            if current_line_length >= width as usize {
                line_count += 1;
                current_line_length = 0; // Reset for the next line
            }
        }

        // Stop if we have reached the max_lines
        if line_count >= max_lines {
            break;
        }
    }

    result
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
