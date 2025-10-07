use crate::file;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::Position;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::{DefaultTerminal, Frame};

pub struct App {
    pub path: std::path::PathBuf,
    pub content: String,
    pub buffer: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
}

impl App {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            content: String::new(),
            buffer: Vec::new(),
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
            scroll_col: 0,
        }
    }

    pub fn load_content(&mut self) {
        self.content = match file::read_file(&self.path) {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        };
        // self.buffer = self.content.split('\n').map(|s| s.to_string()).collect();
        self.buffer = self.content.lines().map(|s| s.to_string()).collect();
    }

    pub fn run(&mut self) -> Result<()> {
        self.load_content();
        // Init ratatui => terminal UI library
        let mut terminal = ratatui::init();
        let result = self.life_cycle(&mut terminal);
        ratatui::restore();
        result
    }

    pub fn life_cycle(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()?
                && key.kind == event::KeyEventKind::Press
            {
                match key.code {
                    KeyCode::Left => {
                        if self.cursor_col > 0 {
                            self.cursor_col -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if self.cursor_col < self.buffer[self.cursor_row].len() {
                            self.cursor_col += 1;
                        }
                    }
                    KeyCode::Up => {
                        if self.cursor_row > 0 {
                            self.cursor_row -= 1;
                            self.cursor_col =
                                self.cursor_col.min(self.buffer[self.cursor_row].len());
                        }
                    }
                    KeyCode::Down => {
                        if self.cursor_row + 1 < self.buffer.len() {
                            self.cursor_row += 1;
                            self.cursor_col =
                                self.cursor_col.min(self.buffer[self.cursor_row].len());
                        }
                    }
                    KeyCode::Char('q') if key.modifiers == KeyModifiers::CONTROL => {
                        break Ok(());
                    }
                    KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                        file::write_file(&self.path, self.buffer.join("\n"))?;
                    }
                    KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                        self.buffer[self.cursor_row].insert(self.cursor_col, c);
                        self.cursor_col += 1;
                    }
                    KeyCode::Backspace => {
                        if self.cursor_col == 0 && self.cursor_row > 0 {
                            let current = self.buffer.remove(self.cursor_row);
                            self.cursor_row -= 1;
                            self.cursor_col = self.buffer[self.cursor_row].len();
                            self.buffer[self.cursor_row].push_str(&current);
                        } else if self.cursor_col > 0 {
                            self.cursor_col -= 1;
                            self.buffer[self.cursor_row].remove(self.cursor_col);
                        }
                    }
                    KeyCode::Enter => {
                        let current = self.buffer[self.cursor_row].split_off(self.cursor_col);
                        self.buffer.insert(self.cursor_row + 1, current);
                        self.cursor_row += 1;
                        self.cursor_col = 0;
                    }
                    _ => {}
                }
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        frame.render_widget(Clear, area); // Clean the prev content

        let lines: Vec<Line> = self
            .buffer
            .iter()
            .map(|l| Line::from(Span::raw(l.clone())))
            .collect();

        let text = Text::from(lines);
        frame.render_widget(Paragraph::new(text), area);

        frame.set_cursor_position(Position::new(
            (self.cursor_col - self.scroll_col) as u16,
            (self.cursor_row - self.scroll_row) as u16,
        ));
    }
}
