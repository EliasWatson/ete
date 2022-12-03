use std::{
    fs::{self, File},
    io::{Stdout, Write},
    path::PathBuf,
};

use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear},
};

#[derive(Debug)]
pub struct TextEditor {
    pub alive: bool,
    pub path: PathBuf,
    pub saved: bool,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub cursor_col_offset: u16,
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
    Front,
    Back,
}

impl TextEditor {
    pub fn open_file(path: PathBuf) -> Result<Self, std::io::Error> {
        if path.exists() {
            let file_contents = fs::read_to_string(path.clone())?;

            Ok(Self {
                alive: true,
                path,
                saved: true,
                lines: file_contents.lines().map(String::from).collect(),
                cursor_row: 0,
                cursor_col: 0,
                cursor_col_offset: 2,
            })
        } else {
            Ok(Self {
                alive: true,
                path,
                saved: false,
                lines: vec![String::new()],
                cursor_row: 0,
                cursor_col: 0,
                cursor_col_offset: 2,
            })
        }
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        match event.code {
            // Save
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => self.save(),

            // Quit if saved
            KeyCode::Esc if self.saved => self.alive = false,

            // Quit without saving
            KeyCode::Char('q') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.alive = false
            }

            // Arrow keys
            KeyCode::Up => self.move_cursor(Direction::Up),
            KeyCode::Right => self.move_cursor(Direction::Right),
            KeyCode::Down => self.move_cursor(Direction::Down),
            KeyCode::Left => self.move_cursor(Direction::Left),

            // Home & end
            KeyCode::Home => self.move_cursor(Direction::Front),
            KeyCode::End => self.move_cursor(Direction::Back),

            // Basic Emacs keys
            KeyCode::Char('a') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_cursor(Direction::Front)
            }
            KeyCode::Char('e') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_cursor(Direction::Back)
            }
            KeyCode::Char('u') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.clear_line()
            }

            // New line
            KeyCode::Enter => self.insert_new_line(),

            // Erase text
            KeyCode::Backspace => self.erase_char(),

            // Write text
            KeyCode::Char(c) => self.insert_char(c),

            // Unknown
            _ => {}
        }

        self.cursor_col_offset = self.get_line_number_width() + 1;
    }

    pub fn render(&self, out: &mut Stdout) -> Result<(), std::io::Error> {
        let line_number_width = self.get_line_number_width();

        execute!(out, cursor::Hide)?;

        for (row, line) in self.lines.iter().enumerate() {
            let Ok(row) = row.try_into() else { break; };

            execute!(
                out,
                cursor::MoveTo(0, row),
                SetForegroundColor(Color::Red),
                Print(format!(
                    "{:width$}",
                    row + 1,
                    width = line_number_width as usize
                )),
                ResetColor,
                cursor::MoveTo(line_number_width + 1, row),
                Print(line)
            )?;
        }

        self.render_toolbar(out)?;

        execute!(out, cursor::Show, ResetColor)?;
        Ok(())
    }

    fn get_line_number_width(&self) -> u16 {
        format!("{}", self.lines.len()).len() as u16
    }

    fn render_toolbar(&self, out: &mut Stdout) -> Result<(), std::io::Error> {
        let (width, height) = terminal::size()?;

        let saved_text = if self.saved { "" } else { "Not Saved!" };

        let path_text = self.path.to_string_lossy().to_string();
        let path_text_col = ((width as usize / 2) - (path_text.len() / 2)) as u16;

        let position_text = format!("{}, {}", self.cursor_col, self.cursor_row);
        let position_text_col = width - 1 - position_text.len() as u16;

        execute!(
            out,
            cursor::MoveTo(0, height - 1),
            SetBackgroundColor(Color::White),
            SetForegroundColor(Color::Black),
            Clear(terminal::ClearType::CurrentLine),
            cursor::MoveTo(1, height - 1),
            Print(saved_text),
            cursor::MoveTo(path_text_col, height - 1),
            Print(path_text),
            cursor::MoveTo(position_text_col, height - 1),
            Print(position_text)
        )?;

        Ok(())
    }

    fn save(&mut self) {
        let mut file = File::create(self.path.clone()).unwrap();

        for line in &self.lines {
            writeln!(file, "{}", line).unwrap();
        }

        self.saved = true;
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.cursor_row = self.cursor_row.saturating_sub(1),
            Direction::Right => self.cursor_col = self.cursor_col.saturating_add(1),
            Direction::Down => self.cursor_row = self.cursor_row.saturating_add(1),
            Direction::Left => self.cursor_col = self.cursor_col.saturating_sub(1),
            Direction::Front => self.cursor_col = 0,
            Direction::Back => self.cursor_col = self.lines[self.cursor_row].len(),
        }

        if self.cursor_row >= self.lines.len() {
            self.cursor_row = self.lines.len() - 1;
        }

        let current_line = &self.lines[self.cursor_row];

        if self.cursor_col > current_line.len() {
            self.cursor_col = current_line.len();
        }
    }

    fn insert_new_line(&mut self) {
        if self.cursor_col == 0 {
            // Beginning of line
            self.lines.insert(self.cursor_row, String::new());
        } else if self.cursor_col == self.lines[self.cursor_row].len() {
            // End of line
            self.lines.insert(self.cursor_row + 1, String::new());
        } else {
            // Middle of line
            let new_line = self.lines[self.cursor_row].split_off(self.cursor_col);
            self.lines.insert(self.cursor_row + 1, new_line);
        }

        self.cursor_row += 1;
        self.cursor_col = 0;

        self.saved = false;
    }

    fn clear_line(&mut self) {
        self.lines[self.cursor_row].clear();
        self.cursor_col = 0;
    }

    fn insert_char(&mut self, c: char) {
        self.lines[self.cursor_row].insert(self.cursor_col, c);
        self.cursor_col += 1;

        self.saved = false;
    }

    fn erase_char(&mut self) {
        if self.cursor_col > 0 {
            self.lines[self.cursor_row].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_col == 0 && self.cursor_row > 0 {
            self.cursor_col = self.lines[self.cursor_row - 1].len();

            let line = self.lines.remove(self.cursor_row);
            self.lines[self.cursor_row - 1] += &line;

            self.cursor_row -= 1;
        } else {
            // At col=0 row=0, so do nothing
            return;
        }

        self.saved = false;
    }
}
