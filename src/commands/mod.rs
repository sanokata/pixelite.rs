pub mod mosaic;
pub mod palette;
pub mod sheet;

pub use mosaic::{MosaicArgs, MosaicMode};
pub use palette::PaletteArgs;
pub use sheet::SheetArgs;

use std::path::PathBuf;

use crate::Result;

pub(crate) fn expand_inputs(inputs: &[String]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for pattern in inputs {
        let mut matched: Vec<PathBuf> = glob::glob(pattern)
            .map_err(|e| format!("invalid glob pattern '{}': {}", pattern, e))?
            .filter_map(|r| r.ok())
            .collect();
        if matched.is_empty() {
            paths.push(PathBuf::from(pattern));
        } else {
            matched.sort();
            paths.extend(matched);
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_inputs_invalid_glob_returns_error() {
        let result = expand_inputs(&["[invalid".to_string()]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("[invalid"), "error should mention the pattern");
    }

    #[test]
    fn test_expand_inputs_no_match_returns_literal() {
        let result = expand_inputs(&["__nonexistent_dir__/*.png".to_string()]).unwrap();
        assert_eq!(result, vec![PathBuf::from("__nonexistent_dir__/*.png")]);
    }
}
