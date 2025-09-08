use anyhow::{Context, Result, bail};
use clap::Parser;
use crossterm::event::{self, Event};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "levorg", version = "0.1", about = "Org-mode editor")]
struct Cli {
    path: Option<String>,
}

struct App {
    path: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Use the values from the command line
    let path = cli.path.unwrap_or_else(|| {
        println!("Sin archivo, buffer vacío");
        "untitled.txt".to_string()
    });
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
        if matches!(event::read()?, Event::Key(_)) {
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

fn read_file(path: &str) -> Result<String> {
    let exists =
        fs::exists(path).with_context(|| format!("Can't check existence of file {}", path))?;
    if !exists {
        bail!("File does not exist");
    }

    let content = fs::read_to_string(path).with_context(|| format!("Can't read file {}", path))?;

    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_does_not_exist() {
        assert!(read_file("does_not_exist.txt").is_err())
    }
}
