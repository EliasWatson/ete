mod text_editor;

use std::io::stdout;

use crate::text_editor::TextEditor;
use crossterm::{
    cursor, event, execute,
    terminal::{self, Clear, ClearType},
    Result,
};

fn main() -> Result<()> {
    let mut out = stdout();

    terminal::enable_raw_mode()?;

    let mut text_editor = TextEditor::new();

    while text_editor.alive {
        execute!(out, Clear(ClearType::All))?;

        text_editor.render(&mut out)?;

        execute!(
            out,
            cursor::MoveTo(
                text_editor.cursor_col.try_into().unwrap_or(u16::MAX),
                text_editor.cursor_row.try_into().unwrap_or(u16::MAX)
            ),
        )?;

        match event::read()? {
            event::Event::Key(event) => text_editor.handle_key(event),
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}
