use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::SavedCard;

use crate::{read, utils::clear_terminal};

pub fn unfinished() {
    let filter = "finished == false & suspended == false".to_string();
    let cards = speki_core::cards_filtered(filter);
    if cards.is_empty() {
        clear_terminal();
        println!("no unfinished cards!");
        read();
        return;
    }

    for card in cards {
        loop {
            let mut card = SavedCard::from_id(&card).unwrap();
            clear_terminal();

            let input: String = Input::new()
                .with_prompt(card.front_text())
                .with_initial_text(card.back_text())
                .allow_empty(true)
                .interact_text()
                .expect("Failed to read input");

            let options = vec!["confirm", "keep editing", "next card", "exit"];
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("save answer and mark card as finished?")
                .items(&options)
                .default(0)
                .interact()
                .expect("Failed to make selection");

            card.set_back_text(&input);

            match selection {
                0 => {
                    card.set_finished(true);
                    break;
                }
                1 => continue,
                2 => break,
                3 => return,
                _ => panic!(),
            }
        }
    }
}
