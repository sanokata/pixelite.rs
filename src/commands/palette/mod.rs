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

    #[test]
    fn test_load_gpl_no_colors_returns_error() {
        let content = "GIMP Palette\nName: empty\nColumns: 0\n#\n";
        let path = std::env::temp_dir().join("test_empty.gpl");
        std::fs::write(&path, content).unwrap();
        let result = Palette::load(&path);
        std::fs::remove_file(&path).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_gpl_skips_incomplete_lines() {
        // Lines with fewer than 3 whitespace-separated tokens must be skipped
        let content = "GIMP Palette\nName: test\n#\n255 0\n128 0 128\tPurple\n";
        let path = std::env::temp_dir().join("test_incomplete.gpl");
        std::fs::write(&path, content).unwrap();
        let palette = Palette::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(palette.0.len(), 1);
        assert_eq!(palette.0[0], Rgba([128, 0, 128, 255]));
    }

    #[test]
    fn test_format_detection_uppercase_extension() {
        assert_eq!(
            PaletteFormat::from_path(Path::new("test.GPL")),
            PaletteFormat::Gpl
        );
        assert_eq!(
            PaletteFormat::from_path(Path::new("test.Gpl")),
            PaletteFormat::Gpl
        );
    }

    #[test]
    fn test_format_detection_no_extension() {
        assert_eq!(
            PaletteFormat::from_path(Path::new("palette")),
            PaletteFormat::Png
        );
    }

    #[test]
    fn test_to_exoquant_colors() {
        let palette = Palette(vec![
            Rgba([255, 0, 128, 200]),
            Rgba([10, 20, 30, 255]),
        ]);
        let exo = palette.to_exoquant_colors();
        assert_eq!(exo.len(), 2);
        assert_eq!((exo[0].r, exo[0].g, exo[0].b, exo[0].a), (255, 0, 128, 200));
        assert_eq!((exo[1].r, exo[1].g, exo[1].b, exo[1].a), (10, 20, 30, 255));
    }

    #[test]
    fn test_save_load_gpl_roundtrip() {
        let original = Palette(vec![
            Rgba([255, 0, 0, 255]),
            Rgba([0, 128, 0, 255]),
            Rgba([0, 0, 200, 255]),
        ]);
        let path = std::env::temp_dir().join("test_roundtrip.gpl");
        original.save(&path).unwrap();
        let loaded = Palette::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.0.len(), 3);
        assert_eq!(loaded.0[0], Rgba([255, 0, 0, 255]));
        assert_eq!(loaded.0[1], Rgba([0, 128, 0, 255]));
        assert_eq!(loaded.0[2], Rgba([0, 0, 200, 255]));
    }

    #[test]
    fn test_save_load_png_roundtrip() {
        let original = Palette(vec![
            Rgba([255, 0, 0, 255]),
            Rgba([0, 255, 0, 255]),
            Rgba([0, 0, 255, 255]),
        ]);
        let path = std::env::temp_dir().join("test_roundtrip_palette.png");
        original.save(&path).unwrap();
        let loaded = Palette::load(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.0.len(), original.0.len());
        for color in &original.0 {
            assert!(loaded.0.contains(color), "missing color {:?}", color);
        }
    }

    #[test]
    fn test_apply_nearest_selects_closest_color() {
        let palette = Palette(vec![
            Rgba([255, 0, 0, 255]),
            Rgba([0, 0, 255, 255]),
        ]);
        let mut buf = ImageBuffer::new(2, 1);
        buf.put_pixel(0, 0, Rgba([200, 0, 0, 255])); // closer to red
        buf.put_pixel(1, 0, Rgba([0, 0, 200, 255])); // closer to blue
        let img = DynamicImage::ImageRgba8(buf);

        let result = palette.apply(img, DitherMethod::None).unwrap().into_rgba8();

        assert_eq!(result.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(result.get_pixel(1, 0).0, [0, 0, 255, 255]);
    }

    #[test]
    fn test_apply_floyd_steinberg_output_in_palette() {
        let palette = Palette(vec![
            Rgba([0, 0, 0, 255]),
            Rgba([255, 255, 255, 255]),
        ]);
        let mut buf = ImageBuffer::new(4, 4);
        for (x, y, pixel) in buf.enumerate_pixels_mut() {
            let v = ((x + y) * 32) as u8;
            *pixel = Rgba([v, v, v, 255]);
        }
        let img = DynamicImage::ImageRgba8(buf);

        let result = palette
            .apply(img, DitherMethod::FloydSteinberg)
            .unwrap()
            .into_rgba8();

        let valid: std::collections::HashSet<[u8; 4]> =
            [[0, 0, 0, 255], [255, 255, 255, 255]].into_iter().collect();
        for pixel in result.pixels() {
            assert!(valid.contains(&pixel.0), "unexpected pixel: {:?}", pixel.0);
        }
    }

    #[test]
    fn test_apply_ordered_output_in_palette() {
        let palette = Palette(vec![
            Rgba([0, 0, 0, 255]),
            Rgba([255, 255, 255, 255]),
        ]);
        let mut buf = ImageBuffer::new(4, 4);
        for pixel in buf.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        let img = DynamicImage::ImageRgba8(buf);

        let result = palette
            .apply(img, DitherMethod::Ordered)
            .unwrap()
            .into_rgba8();

        let valid: std::collections::HashSet<[u8; 4]> =
            [[0, 0, 0, 255], [255, 255, 255, 255]].into_iter().collect();
        for pixel in result.pixels() {
            assert!(valid.contains(&pixel.0), "unexpected pixel: {:?}", pixel.0);
        }
    }

    #[test]
    fn test_floyd_steinberg_single_pixel_wide() {
        // Width=1: the x+1<width and x>0 branches are never taken — should not panic
        let palette = Palette(vec![Rgba([0, 0, 0, 255]), Rgba([255, 255, 255, 255])]);
        let mut buf = ImageBuffer::new(1, 4);
        for pixel in buf.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        let img = DynamicImage::ImageRgba8(buf);
        let result = palette.apply(img, DitherMethod::FloydSteinberg).unwrap();
        assert_eq!(result.dimensions(), (1, 4));
    }

    #[test]
    fn test_floyd_steinberg_single_pixel_tall() {
        // Height=1: the y+1<height branch is never taken — should not panic
        let palette = Palette(vec![Rgba([0, 0, 0, 255]), Rgba([255, 255, 255, 255])]);
        let mut buf = ImageBuffer::new(4, 1);
        for pixel in buf.pixels_mut() {
            *pixel = Rgba([128, 128, 128, 255]);
        }
        let img = DynamicImage::ImageRgba8(buf);
        let result = palette.apply(img, DitherMethod::FloydSteinberg).unwrap();
        assert_eq!(result.dimensions(), (4, 1));
    }

    #[test]
    fn test_extract_with_large_image_uses_sampling() {
        // > 10000 pixels triggers the sampling code path
        let mut buf = ImageBuffer::new(101, 100);
        for (x, _, pixel) in buf.enumerate_pixels_mut() {
            *pixel = Rgba([(x % 256) as u8, 0, 0, 255]);
        }
        let img = DynamicImage::ImageRgba8(buf);
        let result = Palette::extract(&img, 4, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().0.len(), 4);
    }
}
