use crate::file;
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};

pub struct App {
    pub path: std::path::PathBuf,
    pub content: String,
}

impl App {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            path,
            content: String::new(),
        }
    }

    pub fn load_content(&mut self) {
        self.content = match file::read_file(&self.path) {
            Ok(content) => content,
            Err(e) => format!("Error: {}", e),
        };
    }

    pub fn run(&mut self) -> Result<()> {
        self.load_content();
        // Init ratatui => terminal UI library
        let mut terminal = ratatui::init();
        let result = self.lyfe_cycle(&mut terminal);
        ratatui::restore();
        result
    }

    pub fn lyfe_cycle(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(event::KeyEvent {
                code: event::KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            }) = event::read()?
            {
                break Ok(());
            }
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new(self.content.clone()), frame.area());
    }
}
