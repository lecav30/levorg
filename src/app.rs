use crate::file;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Position, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph};
use ratatui::{DefaultTerminal, Frame};

pub struct App {
    pub path: std::path::PathBuf,
    pub content: String,
    pub buffer: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
    pub scroll_col: usize,
    pub dirty: bool,
    pub popup_show: bool,
    pub popup_title: String,
    pub popup_message: String,
    pub status_message: String,
    pub status_color: Color,
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
            dirty: false,
            popup_show: false,
            popup_title: String::new(),
            popup_message: String::new(),
            status_message: String::new(),
            status_color: Color::White,
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

            if self.popup_show {
                if let Event::Key(key) = event::read()?
                    && key.kind == event::KeyEventKind::Press
                {
                    match key.code {
                        KeyCode::Char('n') => {
                            self.popup_show = false;
                            self.popup_message = "".to_string();
                            self.popup_title = "".to_string();
                        }
                        KeyCode::Char('y') => {
                            break Ok(());
                        }
                        _ => {}
                    }
                }
            } else {
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
                            if self.dirty {
                                self.popup_show = true;
                                self.popup_title = "Changes detected".to_string();
                                self.popup_message =
                                    "Do you want to leave without saving?".to_string();
                            } else {
                                break Ok(());
                            }
                        }
                        KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                            self.dirty = false;
                            file::write_file(&self.path, self.buffer.join("\n"))?;
                            self.status_message = "Guardado ✓".to_string();
                            self.status_color = Color::Green;
                        }
                        KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE => {
                            self.dirty = true;
                            self.buffer[self.cursor_row].insert(self.cursor_col, c);
                            self.cursor_col += 1;
                        }
                        KeyCode::Backspace => {
                            self.dirty = true;
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
                            self.dirty = true;
                            let current = self.buffer[self.cursor_row].split_off(self.cursor_col);
                            self.buffer.insert(self.cursor_row + 1, current);
                            self.cursor_row += 1;
                            self.cursor_col = 0;
                        }
                        _ => {}
                    }
                }
            }

            // Auto-scroll
            let height = terminal.size()?.height as usize;

            if self.cursor_row < self.scroll_row {
                self.scroll_row = self.cursor_row;
            } else if self.cursor_row >= self.scroll_row + height {
                self.scroll_row = self.cursor_row - height + 1;
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let height = frame.area().height as usize;
        frame.render_widget(Clear, area); // Clean the prev content

        let lines: Vec<Line> = self
            .buffer
            .iter()
            .skip(self.scroll_row)
            .take(height)
            .map(|l| Line::from(Span::raw(l.clone())))
            .collect();
        let text = Text::from(lines);

        let status_paragraph = Paragraph::new(self.status_message.clone())
            .style(Style::default().fg(self.status_color))
            .alignment(Alignment::Left);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // editor
                Constraint::Length(1), // status bar
            ])
            .split(area);

        frame.render_widget(Paragraph::new(text), layout[0]);
        frame.render_widget(status_paragraph, layout[1]);

        frame.set_cursor_position(Position::new(
            (self.cursor_col - self.scroll_col) as u16,
            (self.cursor_row - self.scroll_row) as u16,
        ));

        if self.popup_show {
            let popup = Block::bordered().title(self.popup_title.to_string());
            let popup_area = self.centered_area(area, 60, 40);
            let popup_inner_area = popup.inner(popup_area);
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(popup_inner_area);
            let buttons_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(popup_layout[1]);
            let message = Paragraph::new(self.popup_message.to_string())
                .alignment(Alignment::Center)
                .block(Block::default().padding(Padding::uniform(1)));
            let confirm_button = Paragraph::new("[Y]es")
                .style(Style::default().fg(Color::LightGreen))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
            let cancel_button = Paragraph::new("[N]o")
                .style(Style::default().fg(Color::Red))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup, popup_area);
            frame.render_widget(message, popup_layout[0]);
            frame.render_widget(confirm_button, buttons_layout[0]);
            frame.render_widget(cancel_button, buttons_layout[1]);
        }
    }

    fn centered_area(&mut self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(vertical[1]);

        horizontal[1]
    }
}
