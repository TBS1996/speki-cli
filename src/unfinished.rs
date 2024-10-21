use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::SavedCard;

use crate::utils::{clear_terminal, notify};

pub fn unfinished() {
    let filter = "finished == false & suspended == false".to_string();
    let cards = speki_core::cards_filtered(filter);
    if cards.is_empty() {
        clear_terminal();
        notify("no unfinished cards");
        return;
    }

    for card in cards {
        loop {
            let mut card = SavedCard::from_id(&card).unwrap();
            clear_terminal();

            let input: String = Input::new()
                .with_prompt(card.print())
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

            match selection {
                0 => {
                    card.set_type_normal(card.print(), input);
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
