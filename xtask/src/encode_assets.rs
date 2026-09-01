//! Rasterize and pack SVG line art into 1bpp firmware bitmaps.

use std::fs;
use std::path::Path;

use image::{imageops, GrayImage, Luma};
use papermono_host::Error;
use resvg::tiny_skia::{Color, Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use sha2::{Digest, Sha256};

const WIDTH: u32 = 360;
const HEIGHT: u32 = 240;
const THRESHOLD: u8 = 200;
const PAD_PCT: u32 = 3;
const EXPECTED_BYTES: usize = (WIDTH as usize * HEIGHT as usize) / 8;

/// Unpack 1bpp MSB-first packed bytes into an 8-bit grayscale image for preview.
fn unpack_preview(blob: &[u8]) -> GrayImage {
    let mut im = GrayImage::from_pixel(WIDTH, HEIGHT, Luma([255]));
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let i = (y * WIDTH + x) as usize;
            if (blob[i / 8] & (0x80 >> (i % 8))) != 0 {
                im.put_pixel(x, y, Luma([0]));
            }
        }
    }
    im
}

/// Encode assets under `firmware/embassy-debug/assets`.
pub fn run(repo_root: &Path) -> Result<(), Error> {
    let assets_dir = repo_root.join("firmware/embassy-debug/assets");
    let svg_path = assets_dir.join("ferris-happy-line-art.svg");
    let sha_path = assets_dir.join("ferris-happy-line-art.svg.sha256");
    let bpp_path = assets_dir.join("ferris.1bpp");
    let inv_bpp_path = assets_dir.join("ferris-inv.1bpp");
    let png_path = assets_dir.join("ferris.png");
    let inv_png_path = assets_dir.join("ferris-inv.png");

    if !svg_path.exists() {
        return Err(Error::Device(format!(
            "source SVG not found: {}",
            svg_path.display()
        )));
    }

    println!("rasterizing {}", svg_path.display());
    let svg_bytes = fs::read(&svg_path)
        .map_err(|e| Error::Device(format!("failed to read {}: {e}", svg_path.display())))?;

    let opt = Options::default();
    let tree = Tree::from_data(&svg_bytes, &opt)
        .map_err(|e| Error::Device(format!("failed to parse SVG: {e}")))?;

    // Rasterize at 300 DPI equivalent (300 / 96 = 3.125 scale factor).
    let scale = 300.0 / 96.0;
    let render_w = (tree.size().width() as f64 * scale).round() as u32;
    let render_h = (tree.size().height() as f64 * scale).round() as u32;

    let mut pixmap = Pixmap::new(render_w, render_h)
        .ok_or_else(|| Error::Device("failed to allocate raster pixmap".into()))?;
    pixmap.fill(Color::WHITE);

    let transform = Transform::from_scale(scale as f32, scale as f32);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Find ink bounding box: any pixel with BT.601 luminance < 250
    let mut min_x = render_w;
    let mut min_y = render_h;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found_ink = false;

    let data = pixmap.data();
    for y in 0..render_h {
        for x in 0..render_w {
            let idx = ((y * render_w + x) * 4) as usize;
            let r = data[idx] as u32;
            let g = data[idx + 1] as u32;
            let b = data[idx + 2] as u32;
            let luma = ((299 * r + 587 * g + 114 * b) / 1000) as u8;
            if luma < 250 {
                found_ink = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }

    if !found_ink {
        return Err(Error::Device("no ink found in rasterized SVG".into()));
    }

    // Add 3% margin padding around ink bounds (matching crop_ink)
    let bw = (max_x + 1) - min_x;
    let bh = (max_y + 1) - min_y;
    let pad = (bw.max(bh) * PAD_PCT) / 100;
    let x0 = min_x.saturating_sub(pad);
    let y0 = min_y.saturating_sub(pad);
    let x1 = (max_x + 1 + pad).min(render_w);
    let y1 = (max_y + 1 + pad).min(render_h);
    let crop_w = x1 - x0;
    let crop_h = y1 - y0;

    let mut cropped = GrayImage::new(crop_w, crop_h);
    for cy in 0..crop_h {
        for cx in 0..crop_w {
            let sx = x0 + cx;
            let sy = y0 + cy;
            let idx = ((sy * render_w + sx) * 4) as usize;
            let r = data[idx] as u32;
            let g = data[idx + 1] as u32;
            let b = data[idx + 2] as u32;
            let luma = ((299 * r + 587 * g + 114 * b) / 1000) as u8;
            cropped.put_pixel(cx, cy, Luma([luma]));
        }
    }

    // Proportional fit into WIDTH x HEIGHT canvas using Lanczos3
    let cw = crop_w as f64;
    let ch = crop_h as f64;
    let fit_scale = (WIDTH as f64 / cw).min(HEIGHT as f64 / ch);
    let nw = (cw * fit_scale).round().max(1.0) as u32;
    let nh = (ch * fit_scale).round().max(1.0) as u32;

    let fitted = imageops::resize(&cropped, nw, nh, imageops::FilterType::Lanczos3);

    let mut canvas = GrayImage::from_pixel(WIDTH, HEIGHT, Luma([255]));
    let offset_x = (WIDTH - nw) / 2;
    let offset_y = (HEIGHT - nh) / 2;
    imageops::overlay(&mut canvas, &fitted, offset_x as i64, offset_y as i64);

    // Threshold at 200: 0 if < 200 else 255
    for pixel in canvas.pixels_mut() {
        pixel[0] = if pixel[0] < THRESHOLD { 0 } else { 255 };
    }

    // Pack 1bpp MSB-first: 1 = ink (black)
    let mut packed = vec![0u8; EXPECTED_BYTES];
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if canvas.get_pixel(x, y)[0] < 128 {
                let i = (y * WIDTH + x) as usize;
                packed[i / 8] |= 0x80 >> (i % 8);
            }
        }
    }

    let inverted: Vec<u8> = packed.iter().map(|b| !b).collect();

    // Generate preview PNGs
    let preview_normal = unpack_preview(&packed);
    let preview_inv = unpack_preview(&inverted);

    fs::write(&bpp_path, &packed)
        .map_err(|e| Error::Device(format!("failed to write {}: {e}", bpp_path.display())))?;
    fs::write(&inv_bpp_path, &inverted)
        .map_err(|e| Error::Device(format!("failed to write {}: {e}", inv_bpp_path.display())))?;

    preview_normal
        .save(&png_path)
        .map_err(|e| Error::Device(format!("failed to save {}: {e}", png_path.display())))?;
    preview_inv
        .save(&inv_png_path)
        .map_err(|e| Error::Device(format!("failed to save {}: {e}", inv_png_path.display())))?;

    let svg_hash = format!("{:x}", Sha256::digest(&svg_bytes));
    fs::write(&sha_path, format!("{svg_hash}\n"))
        .map_err(|e| Error::Device(format!("failed to write {}: {e}", sha_path.display())))?;

    println!("encoded 1bpp assets in {}", assets_dir.display());
    println!("sha256: {svg_hash}");
    Ok(())
}
