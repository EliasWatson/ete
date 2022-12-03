use std::{
    fs,
    io::{stdout, Stdout},
    path::PathBuf,
};

use crossterm::{
    cursor,
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
};

#[derive(Debug)]
pub struct TextEditor {
    pub alive: bool,
    pub path: Option<PathBuf>,
    pub saved: bool,
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl TextEditor {
    pub fn new() -> Self {
        Self {
            alive: true,
            path: None,
            saved: true,
            lines: vec![String::new()],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    pub fn open_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let file_contents = fs::read_to_string(path.clone())?;
        let lines: Vec<String> = file_contents.lines().map(String::from).collect();

        Ok(Self {
            alive: true,
            path: Some(path),
            saved: true,
            lines,
            cursor_row: 0,
            cursor_col: 0,
        })
    }

    pub fn handle_key(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('s') if event.modifiers.contains(KeyModifiers::CONTROL) => self.save(),
            KeyCode::Esc => self.alive = false,
            KeyCode::Up => self.move_cursor(Direction::Up),
            KeyCode::Right => self.move_cursor(Direction::Right),
            KeyCode::Down => self.move_cursor(Direction::Down),
            KeyCode::Left => self.move_cursor(Direction::Left),
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Enter => self.insert_new_line(),
            KeyCode::Backspace => self.erase_char(),
            _ => {}
        }
    }

    pub fn render(&self, out: &mut Stdout) -> Result<(), std::io::Error> {
        execute!(out, cursor::Hide)?;

        for (row, line) in self.lines.iter().enumerate() {
            let Ok(row) = row.try_into() else { break; };

            execute!(out, cursor::MoveTo(0, row), Print(line))?;
        }

        execute!(out, cursor::Show)?;
        Ok(())
    }

    fn save(&mut self) {
        // TODO
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.cursor_row = self.cursor_row.saturating_sub(1),
            Direction::Right => self.cursor_col = self.cursor_col.saturating_add(1),
            Direction::Down => self.cursor_row = self.cursor_row.saturating_add(1),
            Direction::Left => self.cursor_col = self.cursor_col.saturating_sub(1),
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
