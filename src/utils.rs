use dialoguer::{theme::ColorfulTheme, Input, Select};
use speki_core::{
    card::AnyType,
    categories::Category,
    common::CardId,
    concept::{Attribute, AttributeId, Concept, ConceptId},
    SavedCard,
};

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

pub fn select_from_all_concepts() -> Option<ConceptId> {
    enumselector::select_item_with_formatter(Concept::load_all(), |concept: &Concept| {
        concept.name.clone()
    })?
    .id
    .into()
}

pub fn select_from_attributes(concept: ConceptId) -> Option<AttributeId> {
    enumselector::select_item_with_formatter(
        Attribute::load_from_concept(concept),
        |attr: &Attribute| attr.pattern().to_owned(),
    )?
    .id
    .into()
}

pub fn select_from_cards(cards: Vec<CardId>) -> Option<CardId> {
    let cards: Vec<SavedCard<AnyType>> = cards
        .into_iter()
        .map(|id| SavedCard::from_id(&id).unwrap())
        .collect();

    enumselector::select_item_with_formatter(cards, |card: &SavedCard<AnyType>| {
        card.print().to_owned()
    })?
    .id()
    .into()
}

pub fn select_from_all_cards() -> Option<CardId> {
    enumselector::select_item_with_formatter(
        SavedCard::load_all_cards(),
        |card: &SavedCard<AnyType>| card.print().to_owned(),
    )?
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

mod cli_justify {

    fn split_at_char(s: &str, n: usize) -> (&str, Option<&str>) {
        for (char_index, (i, _)) in s.char_indices().enumerate() {
            if char_index == n {
                let (w1, w2) = s.split_at(i);
                return (w1, Some(w2));
            }
        }

        (s, None)
    }

    pub fn justify(text: &str, line_width: usize) -> Vec<String> {
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut lines: Vec<String> = Vec::new();

        for paragraph in paragraphs {
            let raw_words: Vec<&str> = paragraph.split_whitespace().collect();
            let mut words = vec![];

            for mut word in raw_words {
                while let (w1, Some(w2)) = split_at_char(word, line_width) {
                    words.push(w1);
                    word = w2;
                }

                words.push(word);
            }

            let mut line: Vec<&str> = Vec::new();
            let mut len = 0;

            for word in words {
                if len + word.len() > line_width {
                    lines.push(justify_line(&line, line_width));
                    line.clear();
                    len = 0;
                }
                line.push(word);
                len += word.len() + 1; // +1 for space
            }

            // Add the last line of the paragraph
            if !line.is_empty() {
                lines.push(line.join(" "));
            }

            // Add a blank line after each paragraph to preserve paragraph breaks
            lines.push(String::new());
        }

        lines
    }

    fn justify_line(line: &[&str], line_width: usize) -> String {
        let word_len: usize = line.iter().map(|s| s.len()).sum();
        let spaces = line_width as i64 - word_len as i64;
        let spaces = spaces.max(0) as usize;

        let line_len_div = if (line.len() > 1) {
            (line.len() - 1)
        } else {
            1
        };

        let each_space = spaces / line_len_div;
        let extra_space = spaces % line_len_div;

        let mut justified = String::new();
        for (i, word) in line.iter().enumerate() {
            justified.push_str(word);
            if i < line.len() - 1 {
                let mut space = " ".repeat(each_space);
                if i < extra_space {
                    space.push(' ');
                }
                justified.push_str(&space);
            }
        }

        justified
    }
}
