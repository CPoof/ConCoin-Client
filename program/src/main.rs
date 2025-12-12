use rand::rngs::OsRng;
use rand::rand_core::TryRngCore;

use sha2::{Digest, Sha512};
use std::io::{self};
use hex;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Flex, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    prelude::{Rect},
    widgets::{Block, Clear, List, ListItem, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded hashes
    hash: Vec<String>,
    // Stores secret input and pepper
    secrets: Vec<Vec<String>>, 

    show_saved_popup: bool,
    //Info shows detail, result returns Success or Failure
    saved_popup_info: String,
    saved_popup_result: String,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            hash: Vec::new(),
            secrets: Vec::new(),
            character_index: 0,

            show_saved_popup: false,
            saved_popup_info: String::new(),
            saved_popup_result: String::new(),
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_input(&mut self) {
        fn generate_secure_pepper() -> Vec<u8> {
            let mut os_rng = OsRng;
    
            let mut pepper_bytes = [0u8; 32];
    
            os_rng.try_fill_bytes(&mut pepper_bytes)
                .expect("Fatal Error: OS RNG failed to provide secure random bytes.");

            return pepper_bytes.to_vec();
        }

        fn hash_input_with_pepper(input: &str, pepper: &[u8]) -> String {
            let mut hasher = Sha512::new();

            hasher.update(pepper);        
            hasher.update(input.as_bytes());  

            let result = hex::encode(hasher.finalize());

            // Return the final hex-encoded hash string
            return result
        }

        // Add hash to list of hashes to display
        let pepper= generate_secure_pepper();
        let hashed_input = hash_input_with_pepper(&self.input, &pepper);
        self.hash.push(hashed_input);

        //Add salt and input to secrets
        let mut combined_pepper_input: String = hex::encode(pepper);
        combined_pepper_input.push_str(",");
        combined_pepper_input.push_str(&hex::encode(&self.input));
        self.secrets.push(vec![combined_pepper_input]);

        self.input.clear();
        self.reset_cursor();
    }

    fn save_data_to_json(data: Vec<Vec<String>>, filename: &str) -> io::Result<()> {
        // Convert the struct to a pretty JSON string
        let json_string = serde_json::to_string_pretty(&data)?;

        // Save the file in the current directory
        std::fs::write(filename, json_string)?;
    
        Ok(())
    }
    fn handle_save(&mut self) {
        let filename = "secrets.json";
        match App::save_data_to_json(self.secrets.clone(), filename) {
            Ok(_) => {
                self.saved_popup_info = format!("Data successfully saved to {:?}. Press x to close.", filename);
                self.saved_popup_result = "Success".to_string();
                self.show_saved_popup = true;
            }
            Err(e) => {
                self.saved_popup_info = format!("Error saving file: {:?}. Press x to close", e);
                self.saved_popup_result = "Failure".to_string();
                self.show_saved_popup = true;
            }
        }
    }
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('s') => {
                            App::handle_save(&mut self);
                        }
                        KeyCode::Char('x') => {
                            self.show_saved_popup = false;
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_input(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, hash_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing. ".bold(),
                    "Press ".into(),
                    "s".bold(),
                    " to save".bold(),
                    " secret values to a json file".into()
                ],
                Style::default(),
                //Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to hash the input".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let hash: Vec<ListItem> = self
            .hash
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let hash = List::new(hash).block(Block::bordered().title("Hash"));
        frame.render_widget(hash, hash_area);

        fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
            let [area] = Layout::horizontal([horizontal])
            .flex(Flex::Center)
            .areas(area);
            let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
            area
        }

        if self.show_saved_popup{
            let area = center(
                frame.area(),
                Constraint::Percentage(25),
                Constraint::Length(5), // top and bottom border + content
            );
            let popup = Paragraph::new(self.saved_popup_info.clone())
                                        .block(Block::bordered().title(self.saved_popup_result.clone())).wrap(Wrap { trim: false });
            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
    }
}

/* 
fn main() {
    let mut hasher = Sha512::new();
    hasher.update(b"hello world");
    
    let result = hex::encode(hasher.finalize());

    println!("{result}");
}*/
