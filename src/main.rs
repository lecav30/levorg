use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "levorg", version = "0.1", about = "Org-mode editor")]
struct Cli {
    path: Option<std::path::PathBuf>,
}

struct App {
    path: std::path::PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Use the values from the command line
    let path = cli
        .path
        .unwrap_or_else(|| ".".into())
        .canonicalize()
        .unwrap_or_else(|_| ".".into());
    let mut app = App { path };

    // Init ratatui => terminal UI library
    let terminal = ratatui::init();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

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

fn render(frame: &mut Frame, app: &App) {
    let text = match read_file(&app.path) {
        Ok(content) => content,
        Err(e) => format!("Error: {}", e),
    };
    frame.render_widget(Paragraph::new(text), frame.area());
}

fn read_file(path: &std::path::Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Can't read file {}", path.to_string_lossy()))?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_does_not_exist() {
        let path = std::path::Path::new("does_not_exist.txt");
        assert!(read_file(path).is_err());
    }
}
