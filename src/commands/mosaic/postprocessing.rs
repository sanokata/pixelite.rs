use image::{DynamicImage, Rgba};
use std::collections::HashMap;

use crate::Result;

use super::MosaicMode;

/// Post-processes the image after quantization based on the selected generation mode.
///
/// This pipeline applies a series of filters to refine the pixel art quality,
/// such as removing isolated pixels or fixing staircase patterns.
pub fn postprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Character => {
            let img = remove_orphan_pixels(img)?;
            let img = apply_pixel_perfection(img)?;
            let img = add_outlines(img)?;
            let img = remove_anti_aliasing(img)?;
            Ok(img)
        }
        MosaicMode::Background => {
            let img = remove_anti_aliasing(img)?;
            Ok(img)
        }
    }
}

/// Removes isolated "orphan" pixels by replacing them with the majority color of their neighbors.
/// A pixel is considered an orphan if none of its 8 neighbors have the same color.
fn remove_orphan_pixels(img: DynamicImage) -> Result<DynamicImage> {
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 0..height {
        for x in 0..width {
            if !is_orphan(&rgba, x, y) {
                continue;
            }

            let majority_color = find_majority_color(&rgba, x, y);
            output.put_pixel(x, y, majority_color);
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Corrects "staircase" patterns (L-shapes) to make diagonal lines look smoother.
/// This is a common technique in manual pixel art cleaning.
fn apply_pixel_perfection(img: DynamicImage) -> Result<DynamicImage> {
    apply_filter(img, |rgba, x, y, p, [u, d, l, r, ul, ur, dl, dr]| {
        let targets = [
            (l, u, r, d, ul), // Top-left L
            (r, u, l, d, ur), // Top-right L
            (l, d, r, u, dl), // Bottom-left L
            (r, d, l, u, dr), // Bottom-right L
        ];

        targets.into_iter().find_map(|(adj1, adj2, opp1, opp2, diag)| {
            (adj1 == p && adj2 == p && opp1 != p && opp2 != p && diag != p).then(|| {
                if opp1 == opp2 {
                    *opp1
                } else {
                    let majority = find_majority_color(rgba, x, y);
                    if majority != *p { majority } else { *opp1 }
                }
            })
        })
    })
}

/// Adds a 1-pixel black outline to any opaque pixel that is adjacent to a transparent area.
/// This is specifically used in Character mode to make the character stand out.
fn add_outlines(img: DynamicImage) -> Result<DynamicImage> {
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    let outline_color = Rgba([0, 0, 0, 255]);

    for y in 0..height {
        for x in 0..width {
            if rgba.get_pixel(x, y)[3] > 0 {
                continue;
            }

            let is_outline_pixel = (y > 0 && rgba.get_pixel(x, y - 1)[3] > 0)
                || (y + 1 < height && rgba.get_pixel(x, y + 1)[3] > 0)
                || (x > 0 && rgba.get_pixel(x - 1, y)[3] > 0)
                || (x + 1 < width && rgba.get_pixel(x + 1, y)[3] > 0);

            if is_outline_pixel {
                output.put_pixel(x, y, outline_color);
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Removes anti-aliasing artifacts by detecting pixels that are sandwiched between
/// two neighbors of the same color and replacing the middle pixel with that color.
fn remove_anti_aliasing(img: DynamicImage) -> Result<DynamicImage> {
    apply_filter(img, |_rgba, _x, _y, p, [u, d, l, r, ul, ur, dl, dr]| {
        [(l, r), (u, d), (ul, dr), (ur, dl)]
            .into_iter()
            .find_map(|(n1, n2)| (n1 == n2 && n1 != p).then(|| *n1))
    })
}

/// A generic utility to apply a pattern-matching filter across the image.
/// Skips the 1-pixel border of the image as it requires a full 3x3 neighborhood.
fn apply_filter<F>(img: DynamicImage, filter: F) -> Result<DynamicImage>
where
    F: Fn(&image::RgbaImage, u32, u32, &Rgba<u8>, [&Rgba<u8>; 8]) -> Option<Rgba<u8>>,
{
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let p = rgba.get_pixel(x, y);
            if let Some(neighbors) = get_neighbors(&rgba, x, y) {
                if let Some(new_color) = filter(&rgba, x, y, p, neighbors) {
                    output.put_pixel(x, y, new_color);
                }
            }
        }
    }

    Ok(DynamicImage::ImageRgba8(output))
}

/// Checks if a pixel at (x, y) has no neighbors of the same color.
fn is_orphan(img: &image::RgbaImage, x: u32, y: u32) -> bool {
    let (width, height) = img.dimensions();
    let center_pixel = img.get_pixel(x, y);

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                if img.get_pixel(nx as u32, ny as u32) == center_pixel {
                    return false;
                }
            }
        }
    }
    true
}

/// Finds the most frequent color among the 8 neighbors of the pixel at (x, y).
fn find_majority_color(img: &image::RgbaImage, x: u32, y: u32) -> Rgba<u8> {
    let (width, height) = img.dimensions();
    let mut counts: HashMap<Rgba<u8>, u32> = HashMap::with_capacity(8);

    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = x as i32 + dx;
            let ny = y as i32 + dy;

            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let neighbor_pixel = img.get_pixel(nx as u32, ny as u32);
                *counts.entry(*neighbor_pixel).or_insert(0) += 1;
            }
        }
    }

    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(c, _)| c)
        .unwrap_or_else(|| *img.get_pixel(x, y))
}

/// Retrieves the 8 neighboring pixels [u, d, l, r, ul, ur, dl, dr].
#[inline]
fn get_neighbors(img: &image::RgbaImage, x: u32, y: u32) -> Option<[&Rgba<u8>; 8]> {
    let (width, height) = img.dimensions();
    if x == 0 || y == 0 || x >= width - 1 || y >= height - 1 {
        return None;
    }

    Some([
        img.get_pixel(x, y - 1),     // u
        img.get_pixel(x, y + 1),     // d
        img.get_pixel(x - 1, y),     // l
        img.get_pixel(x + 1, y),     // r
        img.get_pixel(x - 1, y - 1), // ul
        img.get_pixel(x + 1, y - 1), // ur
        img.get_pixel(x - 1, y + 1), // dl
        img.get_pixel(x + 1, y + 1), // dr
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn create_test_img(pixels: Vec<Vec<[u8; 4]>>) -> DynamicImage {
        let height = pixels.len() as u32;
        let width = pixels[0].len() as u32;
        let mut buf = ImageBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                buf.put_pixel(x, y, Rgba(pixels[y as usize][x as usize]));
            }
        }
        DynamicImage::ImageRgba8(buf)
    }

    #[test]
    fn test_remove_orphan_pixels() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let img = create_test_img(vec![
            vec![white, white, white],
            vec![white, black, white],
            vec![white, white, white],
        ]);
        let result = remove_orphan_pixels(img).unwrap().into_rgba8();
        // The middle black pixel should be replaced by white
        assert_eq!(result.get_pixel(1, 1).0, white);
    }

    #[test]
    fn test_remove_anti_aliasing() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let gray = [128, 128, 128, 255];
        let img = create_test_img(vec![
            vec![white, black, white],
            vec![white, gray,  white], // sandwiched horizontally
            vec![white, black, white],
        ]);
        let result = remove_anti_aliasing(img).unwrap().into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, white);
    }

    #[test]
    fn test_add_outlines() {
        let transparent = [0, 0, 0, 0];
        let red = [255, 0, 0, 255];
        let black = [0, 0, 0, 255];
        let img = create_test_img(vec![
            vec![transparent, transparent, transparent],
            vec![transparent, red,         transparent],
            vec![transparent, transparent, transparent],
        ]);
        let result = add_outlines(img).unwrap().into_rgba8();
        // The red pixel (1,1) should stay red, but neighbors should become black
        assert_eq!(result.get_pixel(1, 1).0, red);
        assert_eq!(result.get_pixel(1, 0).0, black);
        assert_eq!(result.get_pixel(0, 1).0, black);
    }

    #[test]
    fn test_apply_pixel_perfection() {
        let w = [255, 255, 255, 255];
        let b = [0, 0, 0, 255];
        // L-shape of white pixels in a black background
        // B B B B
        // B W W B
        // B W B B
        // B B B B
        let img = create_test_img(vec![
            vec![b, b, b, b],
            vec![b, w, w, b],
            vec![b, w, b, b],
            vec![b, b, b, b],
        ]);
        let result = apply_pixel_perfection(img).unwrap().into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, b);
    }
}
