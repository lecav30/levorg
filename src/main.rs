mod app;
mod file;

use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "levorg", version = "0.1", about = "Org-mode editor")]
struct Cli {
    path: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // Use the values from the command line
    let path = cli
        .path
        .unwrap_or_else(|| ".".into())
        .canonicalize()
        .unwrap_or_else(|_| ".".into());

    let mut app = app::App::new(path);
    app.run()
}
