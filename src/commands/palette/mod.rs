pub mod apply;
pub mod extract;

use crate::Result;
use clap::{Args, Subcommand, ValueEnum};
use exoquant::{Color, Colorf, convert_to_indexed, ditherer, optimizer};
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba};
use std::path::Path;

/// A structure representing a color palette.
#[derive(Debug, Clone)]
pub struct Palette(pub Vec<Rgba<u8>>);

#[derive(Debug, PartialEq)]
enum PaletteFormat {
    Gpl,
    Png,
}

/// Supported dithering methods
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum DitherMethod {
    /// No dithering (Nearest neighbor)
    None,
    /// Floyd-Steinberg error diffusion
    FloydSteinberg,
    /// Ordered dithering using a 4x4 Bayer matrix
    Ordered,
}

impl PaletteFormat {
    fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
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

        Ok(Palette(
            palette
                .into_iter()
                .map(|c| Rgba([c.r, c.g, c.b, c.a]))
                .collect(),
        ))
    }

    /// Applies this palette to an image using the specified dithering method.
    pub fn apply(&self, img: DynamicImage, method: DitherMethod) -> Result<DynamicImage> {
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

        match method {
            DitherMethod::None => Ok(self.apply_nearest(img, &exo_palette, &exo_palette_f)),
            DitherMethod::FloydSteinberg => {
                Ok(self.apply_floyd_steinberg(img, &exo_palette, &exo_palette_f))
            }
            DitherMethod::Ordered => Ok(self.apply_ordered(img, &exo_palette, &exo_palette_f)),
        }
    }

    fn apply_nearest(
        &self,
        img: DynamicImage,
        exo_palette: &[Color],
        exo_palette_f: &[Colorf],
    ) -> DynamicImage {
        let (width, height) = img.dimensions();
        let quantized = ImageBuffer::from_fn(width, height, |x, y| {
            let p = img.get_pixel(x, y);
            let pf = Colorf {
                r: p[0] as f64,
                g: p[1] as f64,
                b: p[2] as f64,
                a: p[3] as f64,
            };
            let best_idx = self.find_nearest(&pf, exo_palette_f);
            let c = exo_palette[best_idx];
            Rgba([c.r, c.g, c.b, c.a])
        });
        DynamicImage::ImageRgba8(quantized)
    }

    fn apply_floyd_steinberg(
        &self,
        img: DynamicImage,
        exo_palette: &[Color],
        exo_palette_f: &[Colorf],
    ) -> DynamicImage {
        let (width, height) = img.dimensions();
        let mut output = ImageBuffer::new(width, height);

        let mut curr_error = vec![
            Colorf {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            };
            width as usize
        ];
        let mut next_error = vec![
            Colorf {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0
            };
            width as usize
        ];

        for y in 0..height {
            for x in 0..width {
                let p = img.get_pixel(x, y);
                let err = curr_error[x as usize];

                let pf = Colorf {
                    r: (p[0] as f64 + err.r).clamp(0.0, 255.0),
                    g: (p[1] as f64 + err.g).clamp(0.0, 255.0),
                    b: (p[2] as f64 + err.b).clamp(0.0, 255.0),
                    a: (p[3] as f64 + err.a).clamp(0.0, 255.0),
                };

                let best_idx = self.find_nearest(&pf, exo_palette_f);
                let c = exo_palette[best_idx];
                output.put_pixel(x, y, Rgba([c.r, c.g, c.b, c.a]));

                let diff = Colorf {
                    r: pf.r - c.r as f64,
                    g: pf.g - c.g as f64,
                    b: pf.b - c.b as f64,
                    a: pf.a - c.a as f64,
                };

                if x + 1 < width {
                    curr_error[(x + 1) as usize].r += diff.r * 7.0 / 16.0;
                    curr_error[(x + 1) as usize].g += diff.g * 7.0 / 16.0;
                    curr_error[(x + 1) as usize].b += diff.b * 7.0 / 16.0;
                    curr_error[(x + 1) as usize].a += diff.a * 7.0 / 16.0;
                }
                if y + 1 < height {
                    if x > 0 {
                        next_error[(x - 1) as usize].r += diff.r * 3.0 / 16.0;
                        next_error[(x - 1) as usize].g += diff.g * 3.0 / 16.0;
                        next_error[(x - 1) as usize].b += diff.b * 3.0 / 16.0;
                        next_error[(x - 1) as usize].a += diff.a * 3.0 / 16.0;
                    }
                    next_error[x as usize].r += diff.r * 5.0 / 16.0;
                    next_error[x as usize].g += diff.g * 5.0 / 16.0;
                    next_error[x as usize].b += diff.b * 5.0 / 16.0;
                    next_error[x as usize].a += diff.a * 5.0 / 16.0;
                    if x + 1 < width {
                        next_error[(x + 1) as usize].r += diff.r * 1.0 / 16.0;
                        next_error[(x + 1) as usize].g += diff.g * 1.0 / 16.0;
                        next_error[(x + 1) as usize].b += diff.b * 1.0 / 16.0;
                        next_error[(x + 1) as usize].a += diff.a * 1.0 / 16.0;
                    }
                }
            }
            std::mem::swap(&mut curr_error, &mut next_error);
            for err in next_error.iter_mut() {
                *err = Colorf {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                };
            }
        }

        DynamicImage::ImageRgba8(output)
    }

    fn apply_ordered(
        &self,
        img: DynamicImage,
        exo_palette: &[Color],
        exo_palette_f: &[Colorf],
    ) -> DynamicImage {
        let (width, height) = img.dimensions();
        // 4x4 Bayer Matrix
        let bayer: [[f64; 4]; 4] = [
            [0.0, 8.0, 2.0, 10.0],
            [12.0, 4.0, 14.0, 6.0],
            [3.0, 11.0, 1.0, 9.0],
            [15.0, 7.0, 13.0, 5.0],
        ];

        let quantized = ImageBuffer::from_fn(width, height, |x, y| {
            let p = img.get_pixel(x, y);
            // Bayer matrix affects the color by adding a positional offset
            // Scale the 0-15 matrix value to a centered threshold: (value - 7.5) * (255 / 16)
            let threshold = (bayer[(y % 4) as usize][(x % 4) as usize] - 7.5) * (255.0 / 16.0);

            let pf = Colorf {
                r: (p[0] as f64 + threshold).clamp(0.0, 255.0),
                g: (p[1] as f64 + threshold).clamp(0.0, 255.0),
                b: (p[2] as f64 + threshold).clamp(0.0, 255.0),
                a: p[3] as f64, // Usually we don't dither alpha
            };

            let best_idx = self.find_nearest(&pf, exo_palette_f);
            let c = exo_palette[best_idx];
            Rgba([c.r, c.g, c.b, c.a])
        });

        DynamicImage::ImageRgba8(quantized)
    }

    fn find_nearest(&self, p: &Colorf, palette: &[Colorf]) -> usize {
        let mut best_idx = 0;
        let mut min_dist = f64::MAX;
        for (i, cf) in palette.iter().enumerate() {
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
        best_idx
    }

    pub fn load(path: &Path) -> Result<Self> {
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        if extension == "gpl" {
            Self::load_from_gpl(path)
        } else {
            Self::load_from_png(path)
        }
    }

    fn load_from_gpl(path: &Path) -> Result<Self> {
        use std::io::{BufRead, BufReader};
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut colors = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#')
                || line.is_empty()
                || line.starts_with("GIMP")
                || line.starts_with("Name")
                || line.starts_with("Columns")
            {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let r = parts[0].parse::<u8>().unwrap_or(0);
                let g = parts[1].parse::<u8>().unwrap_or(0);
                let b = parts[2].parse::<u8>().unwrap_or(0);
                colors.push(Rgba([r, g, b, 255]));
            }
        }
        if colors.is_empty() {
            return Err("No colors found in GPL file".into());
        }
        Ok(Palette(colors))
    }

    fn load_from_png(path: &Path) -> Result<Self> {
        let img = image::open(path)?;
        let mut colors = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for (_, _, pixel) in img.pixels() {
            if seen.insert(pixel.0) {
                colors.push(pixel);
            }
        }
        Ok(Palette(colors))
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
        writeln!(
            writer,
            "GIMP Palette\nName: {}\nColumns: 0\n#",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("pixelite")
        )?;
        for color in &self.0 {
            writeln!(
                writer,
                "{:>3} {:>3} {:>3}\tUntitled",
                color[0], color[1], color[2]
            )?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn to_exoquant_colors(&self) -> Vec<Color> {
        self.0
            .iter()
            .map(|c| Color::new(c[0], c[1], c[2], c[3]))
            .collect()
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
    /// Apply a palette to an image
    Apply(apply::ApplyArgs),
}

pub fn run(args: PaletteArgs) -> Result<()> {
    match args.command {
        PaletteCommand::Extract(args) => extract::run(args),
        PaletteCommand::Apply(args) => apply::run(args),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_format_detection() {
        assert_eq!(
            PaletteFormat::from_path(Path::new("test.gpl")),
            PaletteFormat::Gpl
        );
        assert_eq!(
            PaletteFormat::from_path(Path::new("test.png")),
            PaletteFormat::Png
        );
    }

    #[test]
    fn test_load_palette_from_gpl() {
        let content = "GIMP Palette\nName: test\n#\n255 0 0\tRed\n0 255 0\tGreen\n";
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_load.gpl");
        std::fs::write(&path, content).unwrap();
        let palette = Palette::load(&path).unwrap();
        assert_eq!(palette.0.len(), 2);
        assert_eq!(palette.0[0], Rgba([255, 0, 0, 255]));
        assert_eq!(palette.0[1], Rgba([0, 255, 0, 255]));
        std::fs::remove_file(path).unwrap();
    }
}
