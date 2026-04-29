pub mod extract;

use std::path::Path;
use clap::{Args, Subcommand};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba, GenericImageView};
use exoquant::{Color, Colorf, ditherer, optimizer, convert_to_indexed};
use crate::Result;

/// A structure representing a color palette.
#[derive(Debug, Clone)]
pub struct Palette(pub Vec<Rgba<u8>>);

#[derive(Debug, PartialEq)]
enum PaletteFormat {
    Gpl,
    Png,
}

impl PaletteFormat {
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("gpl") => PaletteFormat::Gpl,
            _ => PaletteFormat::Png,
        }
    }
}

impl Palette {
    /// Extracts a palette from an image using KMeans.
    pub fn extract(img: &DynamicImage, color_count: u8, use_sampling: bool) -> Result<Self> {
        let (width, height) = img.dimensions();
        let total_pixels = width * height;

        // Memory efficiency: Use img.pixels() directly to avoid full buffer copy
        let pixels: Vec<Color> = if use_sampling && total_pixels > 10000 {
            let step = (total_pixels / 10000).max(1);
            img.pixels()
                .step_by(step as usize)
                .map(|(_, _, p)| Color::new(p[0], p[1], p[2], p[3]))
                .collect()
        } else {
            img.pixels()
                .map(|(_, _, p)| Color::new(p[0], p[1], p[2], p[3]))
                .collect()
        };

        let (palette, _) = convert_to_indexed(
            &pixels,
            width as usize,
            color_count as usize,
            &optimizer::KMeans,
            &ditherer::None,
        );

        Ok(Palette(palette.into_iter().map(|c| Rgba([c.r, c.g, c.b, c.a])).collect()))
    }

    /// Applies this palette to an image.
    /// TODO: implement dithering support
    pub fn apply(&self, img: DynamicImage, _use_dithering: bool) -> DynamicImage {
        let (width, height) = img.dimensions();
        
        // Efficiency: Pre-calculate the float palette and input pixels
        let exo_palette = self.to_exoquant_colors();
        let exo_palette_f: Vec<Colorf> = exo_palette
            .iter()
            .map(|c| Colorf {
                r: c.r as f64,
                g: c.g as f64,
                b: c.b as f64,
                a: c.a as f64,
            })
            .collect();

        let pixels_f: Vec<Colorf> = img.pixels()
            .map(|(_, _, p)| Colorf {
                r: p[0] as f64,
                g: p[1] as f64,
                b: p[2] as f64,
                a: p[3] as f64,
            })
            .collect();

        let quantized = ImageBuffer::from_fn(width, height, |x, y| {
            let p = pixels_f[(y * width + x) as usize];
            let mut best_idx = 0;
            let mut min_dist = f64::MAX;
            
            // Optimization: Iterate over pre-converted float palette
            for (i, cf) in exo_palette_f.iter().enumerate() {
                let dr = p.r - cf.r;
                let dg = p.g - cf.g;
                let db = p.b - cf.b;
                let da = p.a - cf.a;
                let dist = dr * dr + dg * dg + db * db + da * da;
                
                if dist < min_dist {
                    min_dist = dist;
                    best_idx = i;
                }
            }
            
            let color = exo_palette[best_idx];
            Rgba([color.r, color.g, color.b, color.a])
        });

        DynamicImage::ImageRgba8(quantized)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        match PaletteFormat::from_path(path) {
            PaletteFormat::Gpl => self.save_as_gpl(path),
            PaletteFormat::Png => self.save_as_png(path),
        }
    }

    fn save_as_png(&self, path: &Path) -> Result<()> {
        let mut img = ImageBuffer::new(self.0.len() as u32, 1);
        for (x, color) in self.0.iter().enumerate() {
            img.put_pixel(x as u32, 0, *color);
        }
        let format = ImageFormat::from_path(path).unwrap_or(ImageFormat::Png);
        DynamicImage::ImageRgba8(img).save_with_format(path, format)?;
        Ok(())
    }

    fn save_as_gpl(&self, path: &Path) -> Result<()> {
        use std::io::{BufWriter, Write};
        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "GIMP Palette\nName: {}\nColumns: 0\n#", path.file_stem().and_then(|s| s.to_str()).unwrap_or("pixelite"))?;
        for color in &self.0 {
            writeln!(writer, "{:>3} {:>3} {:>3}\tUntitled", color[0], color[1], color[2])?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn to_exoquant_colors(&self) -> Vec<Color> {
        self.0.iter().map(|c| Color::new(c[0], c[1], c[2], c[3])).collect()
    }
}

/// The arguments of palette sub-command
#[derive(Args, Debug)]
pub struct PaletteArgs {
    #[command(subcommand)]
    pub command: PaletteCommand,
}

#[derive(Subcommand, Debug)]
pub enum PaletteCommand {
    /// Extract color palette from an image
    Extract(extract::ExtractArgs),
}

pub fn run(args: PaletteArgs) -> Result<()> {
    match args.command {
        PaletteCommand::Extract(args) => extract::run(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_format_detection() {
        assert_eq!(PaletteFormat::from_path(Path::new("test.gpl")), PaletteFormat::Gpl);
        assert_eq!(PaletteFormat::from_path(Path::new("test.png")), PaletteFormat::Png);
    }
}
