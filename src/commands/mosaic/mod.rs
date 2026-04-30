pub mod postprocessing;
pub mod preprocessing;
pub mod processing;

use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use image::{ImageFormat, open};

use crate::Result;
use crate::commands::palette::DitherMethod;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum MosaicMode {
    #[default]
    Sharp,
    Smooth,
}

/// The arguments of mosaic sub-command
#[derive(Args, Debug)]
#[command(about = "Creates a mosaic-like pixel art from an image.")]
pub struct MosaicArgs {
    /// Input image path(s) or glob pattern(s) (e.g. "sprites/*.png")
    #[arg(short, long, required = true)]
    pub input: Vec<String>,

    /// Output path or pattern. Supports {stem}, {name}, {n} for batch output.
    /// Example: "out/{stem}_pixel.png"
    #[arg(short, long)]
    pub output: String,

    /// The size of the longest edge of the resulting pixel art (e.g., 64). Aspect ratio will be maintained.
    #[arg(short, long, default_value_t = 64)]
    pub size: u32,

    /// The maximum number of colors to use in the output image.
    #[arg(short, long, default_value_t = 16)]
    pub colors: u8,

    /// The mode of pixel art generation.
    #[arg(short = 'm', long, value_enum, default_value_t = MosaicMode::Sharp)]
    pub mode: MosaicMode,

    /// Optional dithering method (overrides mode-based default)
    #[arg(short, long)]
    pub dither: Option<DitherMethod>,

    /// Whether to crop the input image to a square centered on the middle before processing.
    #[arg(long, default_value_t = false)]
    pub crop: bool,

    /// The output scale factor (e.g., 4 results in 4x4 physical pixels per dot).
    #[arg(short = 'S', long, default_value_t = 1)]
    pub scale: u32,
}

pub fn run(args: MosaicArgs) -> Result<()> {
    let inputs = expand_inputs(&args.input)?;
    if inputs.len() > 1 && !args.output.contains('{') {
        return Err(
            "multiple input files require an output pattern containing {stem}, {name}, or {n}"
                .into(),
        );
    }
    for (i, input_path) in inputs.iter().enumerate() {
        let output_path = resolve_output(&args.output, input_path, i + 1);
        process_single(input_path, &output_path, &args)?;
    }
    Ok(())
}

fn expand_inputs(inputs: &[String]) -> Result<Vec<PathBuf>> {
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

fn resolve_output(pattern: &str, input: &Path, n: usize) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let name = input.file_name().and_then(|s| s.to_str()).unwrap_or("output");
    PathBuf::from(
        pattern
            .replace("{stem}", stem)
            .replace("{name}", name)
            .replace("{n}", &n.to_string()),
    )
}

fn process_single(input: &Path, output: &Path, args: &MosaicArgs) -> Result<()> {
    let img = open(input)?;
    // 0. preprocess: noise reduction / alpha normalization per mode
    let preprocessed = preprocessing::preprocess(img, &args.mode)?;
    // 1. crop if requested
    let prepared = preprocessing::crop_to_square(preprocessed, args.crop);
    // 2. resize the original image to the desired size
    let resized = processing::resize_to_max_long_side(prepared, args.size, &args.mode)?;
    // 3. quantize the colors of the resized image to the desired number of colors
    let quantized =
        processing::quantize_colors(resized, args.colors, args.mode.clone(), args.dither.clone())?;
    // 4. postprocess the quantized image
    let postprocessed = postprocessing::postprocess(quantized, &args.mode)?;
    // 5. upscale the result if requested
    let final_image = postprocessing::upscale(postprocessed, args.scale);
    // 6. save the postprocessed image to the output file
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let format = ImageFormat::from_path(output).unwrap_or(ImageFormat::Png);
    final_image.save_with_format(output, format)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_output_stem_strips_extension() {
        let result = resolve_output("out/{stem}.png", Path::new("src/cat.jpg"), 1);
        assert_eq!(result, PathBuf::from("out/cat.png"));
    }

    #[test]
    fn test_resolve_output_name_preserves_extension() {
        let result = resolve_output("out/{name}", Path::new("src/cat.jpg"), 1);
        assert_eq!(result, PathBuf::from("out/cat.jpg"));
    }

    #[test]
    fn test_resolve_output_n_uses_1indexed() {
        let result = resolve_output("out/{n}.png", Path::new("src/cat.png"), 1);
        assert_eq!(result, PathBuf::from("out/1.png"));
    }

    #[test]
    fn test_resolve_output_no_placeholder_passthrough() {
        let result = resolve_output("out/result.png", Path::new("src/cat.png"), 1);
        assert_eq!(result, PathBuf::from("out/result.png"));
    }

    #[test]
    fn test_resolve_output_multiple_placeholders() {
        let result = resolve_output("out/{n}_{stem}.png", Path::new("sprites/hero.png"), 3);
        assert_eq!(result, PathBuf::from("out/3_hero.png"));
    }

    #[test]
    fn test_expand_inputs_invalid_glob_returns_error() {
        let result = expand_inputs(&["[invalid".to_string()]);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("[invalid"), "error should mention the pattern");
    }

    #[test]
    fn test_expand_inputs_no_match_returns_literal() {
        // A well-formed glob that matches nothing falls back to the literal string as a PathBuf
        let result = expand_inputs(&["__nonexistent_dir__/*.png".to_string()]).unwrap();
        assert_eq!(result, vec![PathBuf::from("__nonexistent_dir__/*.png")]);
    }
}
