use image::{DynamicImage, Rgba};

use super::MosaicMode;
use crate::Result;

/// Post-processes the image after quantization based on the selected generation mode.
///
/// This pipeline applies a series of filters to refine the pixel art quality,
/// such as removing isolated pixels or fixing staircase patterns.
pub fn postprocess(img: DynamicImage, mode: &MosaicMode) -> Result<DynamicImage> {
    match mode {
        MosaicMode::Sharp => {
            let img = remove_orphan_pixels(img);
            let img = apply_pixel_perfection(img);
            let img = add_outlines(img);
            let img = remove_anti_aliasing(img);
            Ok(img)
        }
        MosaicMode::Smooth => Ok(remove_anti_aliasing(img)),
    }
}

/// Upscales the image by a given factor using the Nearest Neighbor filter to preserve pixel edges.
pub fn upscale(img: DynamicImage, factor: u32) -> DynamicImage {
    if factor <= 1 {
        return img;
    }
    img.resize(
        img.width() * factor,
        img.height() * factor,
        image::imageops::FilterType::Nearest,
    )
}

/// Removes isolated "orphan" pixels by replacing them with the majority color of their neighbors.
/// A pixel is considered an orphan if none of its 8 neighbors have the same color.
fn remove_orphan_pixels(img: DynamicImage) -> DynamicImage {
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 0..height {
        for x in 0..width {
            if !is_orphan(&rgba, x, y) {
                continue;
            }
            output.put_pixel(x, y, find_majority_color(&rgba, x, y));
        }
    }

    DynamicImage::ImageRgba8(output)
}

/// Corrects "staircase" patterns (L-shapes) to make diagonal lines look smoother.
/// This is a common technique in manual pixel art cleaning.
fn apply_pixel_perfection(img: DynamicImage) -> DynamicImage {
    apply_filter(img, |rgba, x, y, p, [u, d, l, r, ul, ur, dl, dr]| {
        let targets = [
            (l, u, r, d, ul), // Top-left L
            (r, u, l, d, ur), // Top-right L
            (l, d, r, u, dl), // Bottom-left L
            (r, d, l, u, dr), // Bottom-right L
        ];

        targets
            .into_iter()
            .find_map(|(adj1, adj2, opp1, opp2, diag)| {
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
/// This is specifically used in Sharp mode to make the shape stand out.
fn add_outlines(img: DynamicImage) -> DynamicImage {
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

    DynamicImage::ImageRgba8(output)
}

/// Removes anti-aliasing artifacts by detecting pixels that are sandwiched between
/// two neighbors of the same color and replacing the middle pixel with that color.
fn remove_anti_aliasing(img: DynamicImage) -> DynamicImage {
    apply_filter(img, |_rgba, _x, _y, p, [u, d, l, r, ul, ur, dl, dr]| {
        [(l, r), (u, d), (ul, dr), (ur, dl)]
            .into_iter()
            .find_map(|(n1, n2)| (n1 == n2 && n1 != p).then_some(*n1))
    })
}

/// A generic utility to apply a pattern-matching filter across the image.
/// Skips the 1-pixel border of the image as it requires a full 3x3 neighborhood.
fn apply_filter<F>(img: DynamicImage, filter: F) -> DynamicImage
where
    F: Fn(&image::RgbaImage, u32, u32, &Rgba<u8>, [&Rgba<u8>; 8]) -> Option<Rgba<u8>>,
{
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    let mut output = rgba.clone();

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let p = rgba.get_pixel(x, y);
            if let Some(new_color) =
                get_neighbors(&rgba, x, y).and_then(|neighbors| filter(&rgba, x, y, p, neighbors))
            {
                output.put_pixel(x, y, new_color);
            }
        }
    }

    DynamicImage::ImageRgba8(output)
}

/// Calls `f` for each of the (up to 8) in-bounds neighbors of the pixel at (x, y).
fn for_each_neighbor(img: &image::RgbaImage, x: u32, y: u32, mut f: impl FnMut(Rgba<u8>)) {
    let (w, h) = (img.width() as i32, img.height() as i32);
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if (dx, dy) == (0, 0) {
                continue;
            }
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            if nx >= 0 && nx < w && ny >= 0 && ny < h {
                f(*img.get_pixel(nx as u32, ny as u32));
            }
        }
    }
}

/// Checks if a pixel at (x, y) has no neighbors of the same color.
fn is_orphan(img: &image::RgbaImage, x: u32, y: u32) -> bool {
    let center = *img.get_pixel(x, y);
    let mut found = false;
    for_each_neighbor(img, x, y, |p| {
        if p == center {
            found = true;
        }
    });
    !found
}

/// Finds the most frequent color among the (up to 8) neighbors of the pixel at (x, y).
fn find_majority_color(img: &image::RgbaImage, x: u32, y: u32) -> Rgba<u8> {
    // Fixed-size array avoids HashMap overhead for at most 8 entries.
    let mut counts: [(Rgba<u8>, u8); 8] = [(Rgba([0, 0, 0, 0]), 0); 8];
    let mut len = 0usize;

    for_each_neighbor(img, x, y, |p| {
        if let Some(entry) = counts[..len].iter_mut().find(|(c, _)| *c == p) {
            entry.1 += 1;
        } else {
            counts[len] = (p, 1);
            len += 1;
        }
    });

    counts[..len]
        .iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| *c)
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
        let result = remove_orphan_pixels(img).into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, white);
    }

    #[test]
    fn test_upscale_preserves_sharp_edges() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let img = create_test_img(vec![vec![white, black], vec![black, white]]);

        let result = upscale(img, 2).into_rgba8();

        assert_eq!(result.dimensions(), (4, 4));
        assert_eq!(result.get_pixel(0, 0).0, white);
        assert_eq!(result.get_pixel(1, 1).0, white);
        assert_eq!(result.get_pixel(2, 0).0, black);
        assert_eq!(result.get_pixel(3, 1).0, black);
    }

    #[test]
    fn test_remove_anti_aliasing() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let gray = [128, 128, 128, 255];
        let img = create_test_img(vec![
            vec![white, black, white],
            vec![white, gray, white], // sandwiched horizontally
            vec![white, black, white],
        ]);
        let result = remove_anti_aliasing(img).into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, white);
    }

    #[test]
    fn test_add_outlines() {
        let transparent = [0, 0, 0, 0];
        let red = [255, 0, 0, 255];
        let black = [0, 0, 0, 255];
        let img = create_test_img(vec![
            vec![transparent, transparent, transparent],
            vec![transparent, red, transparent],
            vec![transparent, transparent, transparent],
        ]);
        let result = add_outlines(img).into_rgba8();
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
        let result = apply_pixel_perfection(img).into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, b);
    }

    #[test]
    fn test_upscale_factor_one_is_noop() {
        let img = create_test_img(vec![
            vec![[255, 0, 0, 255], [0, 255, 0, 255]],
            vec![[0, 0, 255, 255], [255, 255, 0, 255]],
        ]);
        let result = upscale(img, 1).into_rgba8();
        assert_eq!(result.dimensions(), (2, 2));
        assert_eq!(result.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(result.get_pixel(1, 0).0, [0, 255, 0, 255]);
        assert_eq!(result.get_pixel(0, 1).0, [0, 0, 255, 255]);
        assert_eq!(result.get_pixel(1, 1).0, [255, 255, 0, 255]);
    }

    #[test]
    fn test_postprocess_sharp_mode_completes() {
        let w = [255, 255, 255, 255];
        let t = [0, 0, 0, 0];
        let img = create_test_img(vec![
            vec![t, t, t, t, t],
            vec![t, w, w, w, t],
            vec![t, w, w, w, t],
            vec![t, w, w, w, t],
            vec![t, t, t, t, t],
        ]);
        let result = postprocess(img, &MosaicMode::Sharp).unwrap().into_rgba8();
        // Corner pixels are not 4-connected to any opaque pixel, so no outline is drawn
        assert_eq!(result.get_pixel(0, 0).0, [0, 0, 0, 0]);
        // Transparent pixels directly adjacent to the white block become black outlines
        assert_eq!(result.get_pixel(2, 0).0, [0, 0, 0, 255]);
        assert_eq!(result.get_pixel(0, 2).0, [0, 0, 0, 255]);
        // The center of the white block is untouched by any pipeline step
        assert_eq!(result.get_pixel(2, 2).0, [255, 255, 255, 255]);
        // After outlining, (ur) and (dl) of each white corner become black, forming a matching
        // diagonal pair that remove_anti_aliasing detects — white corners are eroded to black
        assert_eq!(result.get_pixel(1, 1).0, [0, 0, 0, 255]);
    }

    #[test]
    fn test_postprocess_smooth_mode_completes() {
        let w = [255, 255, 255, 255];
        let b = [0, 0, 0, 255];
        let g = [128, 128, 128, 255];
        let img = create_test_img(vec![vec![w, b, w], vec![b, g, b], vec![w, b, w]]);
        let result = postprocess(img, &MosaicMode::Smooth).unwrap().into_rgba8();
        // The gray pixel at (1,1) has black on both its left and right sides;
        // remove_anti_aliasing detects this matching pair and replaces gray with black
        assert_eq!(result.get_pixel(1, 1).0, b);
        // Border pixels lie outside the apply_filter window and must not be modified
        assert_eq!(result.get_pixel(0, 0).0, w);
        assert_eq!(result.get_pixel(1, 0).0, b);
    }

    #[test]
    fn test_remove_orphan_pixels_connected_pixel_unchanged() {
        let red = [255, 0, 0, 255];
        // Center pixel is surrounded by the same color — not an orphan, must not be changed
        let img = create_test_img(vec![
            vec![red, red, red],
            vec![red, red, red],
            vec![red, red, red],
        ]);
        let result = remove_orphan_pixels(img).into_rgba8();
        assert_eq!(result.get_pixel(1, 1).0, red);
    }

    #[test]
    fn test_add_outlines_fully_opaque_unchanged() {
        let red = [255, 0, 0, 255];
        // No transparent pixels means no outline pixels should be added
        let img = create_test_img(vec![
            vec![red, red, red],
            vec![red, red, red],
            vec![red, red, red],
        ]);
        let result = add_outlines(img).into_rgba8();
        for pixel in result.pixels() {
            assert_eq!(pixel.0, red);
        }
    }
}
