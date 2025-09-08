use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn read_file(path: &Path) -> Result<String> {
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
