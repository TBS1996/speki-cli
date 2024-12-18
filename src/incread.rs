use cli_epub_to_text::epub_to_text;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use serde::{Deserialize, Serialize};
use speki_core::common::current_time;
use speki_core::paths::get_share_path;
use std::collections::HashMap;
use std::fs::{self, read_to_string};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::utils::{choose_folder, get_lines};
use crate::utils::{clear_terminal, notify};

pub fn inc_path() -> PathBuf {
    let path = get_share_path().join("texts");
    fs::create_dir_all(&path).unwrap();
    path
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TextFile {
    path: PathBuf,
    position: usize,
    length: usize,
    #[serde(default = "speki_core::common::current_time")]
    time_added: Duration,
}

impl TextFile {
    pub fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        let buffer = 500;

        self.position() + buffer >= self.length
    }

    pub fn avg_daily_progress(&self) -> usize {
        let days_passed = (current_time() - self.time_added).as_secs_f32();
        let avg = self.position() as f32 / days_passed;
        avg as usize
    }

    pub fn progress_percentage(&self) -> f32 {
        (self.position as f32 / self.length as f32) * 100.
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn save_pos(&mut self, new_pos: usize) {
        let mut txt = TextProgress::xload();
        self.position = new_pos;
        txt.0.insert(self.path.clone(), self.clone());
        txt.save();
    }

    pub fn position_decrement(&mut self, dec: usize) {
        let new_pos = if self.position < dec {
            0
        } else {
            self.position - dec
        };

        self.save_pos(new_pos);
    }

    pub fn position_increment(&mut self, inc: usize) {
        let new_pos = self.length.min(self.position + inc);
        self.save_pos(new_pos);
    }

    pub fn load_all() -> Vec<Self> {
        TextProgress::xload()
            .0
            .into_iter()
            .map(|(_, val)| val)
            .collect()
    }

    pub fn load_text(&self) -> String {
        text_load(&self.path).unwrap()
    }
}

fn text_load(p: &Path) -> Option<String> {
    if p.extension().is_some_and(|ext| ext == "pdf") {
        cli_pdf_to_text::pdf_to_text(p.as_os_str().to_str().unwrap()).ok()
    } else if p.extension().is_some_and(|ext| ext == "epub") {
        epub_to_text(p.as_os_str().to_str().unwrap()).ok()
    } else {
        read_to_string(&p).ok()
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct TextProgress(HashMap<PathBuf, TextFile>);

impl TextProgress {
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
                let length = match text_load(&file) {
                    Some(txt) => txt.chars().count(),
                    None => continue,
                };

                txts.0.insert(
                    file.clone(),
                    TextFile {
                        length,
                        path: file,
                        position: 0,
                        time_added: current_time(),
                    },
                );
            }
        }

        txts
    }
}

fn select_text(mut textfiles: Vec<TextFile>) -> TextFile {
    textfiles.sort_by_key(|f| {
        if f.is_finished() {
            usize::MAX
        } else {
            f.avg_daily_progress()
        }
    });

    let named: Vec<String> = textfiles
        .iter()
        .map(|p| format!("{:.1}%: {}", p.progress_percentage(), p.name()))
        .collect();

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("")
        .items(&named)
        .default(0)
        .interact()
        .expect("Failed to make selection");

    textfiles[idx].clone()
}

pub fn textstuff() {
    clear_terminal();
    //    let paths = get_text_files(&inc_path()).unwrap();
    let textfiles = TextFile::load_all();
    if textfiles.is_empty() {
        notify(format!("no available texts. click 'inspect texts' in main menu and add textfiles to get started"));
        return;
    }

    let mut textfile = select_text(textfiles);
    let text = textfile.load_text();
    let category = choose_folder();

    let opts = ["add card", "go forward", "go back", "exit"];
    let mut menu_position = 0;

    let menu_size = opts.len() as u16 + 3;

    loop {
        clear_terminal();
        let (height, width) = console::Term::stdout().size();
        let free_space = if height > menu_size {
            height - menu_size
        } else {
            0
        };

        let line_qty = 20.min(free_space);

        let s = get_lines(
            &text,
            50.min(width as usize),
            line_qty as usize,
            textfile.position(),
        );

        let char_len = s.clone().join("").chars().count();

        for line in s {
            println!("{}", line);
        }

        let idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("")
            .items(&opts)
            .default(menu_position)
            .interact()
            .expect("Failed to make selection");

        menu_position = idx;
        match idx {
            0 => drop(crate::add_cards::add_card(&category)),
            1 => textfile.position_increment(char_len),
            2 => textfile.position_decrement(char_len),
            3 => return,
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
