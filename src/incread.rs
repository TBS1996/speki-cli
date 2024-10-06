use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use serde::{Deserialize, Serialize};
use speki_core::paths::get_share_path;
use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::utils::{choose_folder, get_input, get_lines};
use crate::utils::{clear_terminal, notify};

pub fn inc_path() -> PathBuf {
    let path = get_share_path().join("texts");
    fs::create_dir_all(&path).unwrap();
    path
}

#[derive(Serialize, Deserialize, Default)]
pub struct TextProgress(HashMap<PathBuf, usize>);

impl TextProgress {
    fn get_pos(path: &Path) -> usize {
        let mut selv = Self::load().unwrap_or_default();
        match selv.0.get(path) {
            Some(num) => *num,
            None => {
                selv.0.insert(path.to_path_buf(), 0);
                selv.save();
                0
            }
        }
    }

    fn save_pos(mut self, path: PathBuf, num: usize) -> Self {
        self.0.insert(path, num);
        self.save();
        Self::xload()
    }

    fn save(&self) {
        let s: String = serde_json::to_string_pretty(&self).unwrap();
        let path = Self::path();
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(s.as_bytes()).unwrap();
    }

    fn path() -> PathBuf {
        speki_core::paths::get_share_path().join("bookmarks")
    }

    fn load() -> Option<Self> {
        let s: String = read_to_string(&Self::path()).ok()?;
        serde_json::from_str(&s).ok()
    }

    fn xload() -> Self {
        let mut txts = Self::load().unwrap_or_default();
        let files = get_text_files(&inc_path()).unwrap();

        for file in files {
            if !txts.0.contains_key(&file) {
                txts.0.insert(file.clone(), 0);
            }
        }

        txts
    }
}

fn select_text(paths: Vec<PathBuf>) -> PathBuf {
    let named: Vec<String> = paths
        .clone()
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .items(&named)
        .default(0)
        .interact()
        .expect("Failed to make selection");

    paths[idx].clone()
}

pub fn textstuff() {
    clear_terminal();
    let paths = get_text_files(&inc_path()).unwrap();
    if paths.is_empty() {
        notify(format!("no available texts. click 'inspect texts' in main menu and add textfiles to get started"));
        return;
    }

    let category = choose_folder();

    let file = select_text(paths);

    let window_size = 700;
    let scroll_size = window_size - 200;

    let s: String = file.to_string_lossy().to_owned().to_string();
    let file = PathBuf::from(&s);

    let text: String = read_to_string(&file).unwrap();
    let textlen = text.chars().count();
    let mut txtprog = TextProgress::xload();
    let mut position = TextProgress::get_pos(&file).min(textlen);
    let opts = ["add card", "go forward", "go back", "exit"];
    let mut pos = 0;

    let menu_size = opts.len() as u16 + 3;

    loop {
        clear_terminal();
        //let slice = char_slice(charred.as_slice(), position, window_size);
        //let s: String = slice.iter().collect();
        let (height, width) = console::Term::stdout().size();
        let free_space = if height > menu_size {
            height - menu_size
        } else {
            0
        };

        let line_qty = 20.min(free_space);

        let s = get_lines(&text, 50.min(width as usize), line_qty as usize, position);
        for line in s {
            println!("{}", line);
        }

        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("")
            .items(&opts)
            .default(pos)
            .interact()
            .expect("Failed to make selection");

        match idx {
            0 => {
                pos = 0;
                let input = get_input("");
                if let Some((front, back)) = input.split_once(";") {
                    speki_core::add_card(front.to_string(), back.to_string(), &category);
                } else {
                    speki_core::add_unfinished(input, &category);
                }
            }
            1 => {
                pos = 1;
                position += scroll_size;
                txtprog = txtprog.save_pos(file.clone(), position);
            }
            2 => {
                pos = 2;
                position = if position > scroll_size {
                    position - scroll_size
                } else {
                    0
                };

                txtprog = txtprog.save_pos(file.clone(), position);
            }
            3 => {
                return;
            }
            _ => panic!(),
        }
    }
}

fn get_text_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut text_files = Vec::new();

    if dir.is_dir() {
        let x = fs::read_dir(dir)?;

        for entry in x {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                text_files.push(path);
            }
        }
    }

    Ok(text_files)
}
