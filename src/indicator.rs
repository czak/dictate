//! The indicator shown while a recording is in progress: a small red disc in
//! the top-right corner of the screen.

use anyhow::Result;

use crate::wayland::{Anchor, OverlaySpec, OverlaySurface, ShmBuffer, Wayland};

/// Edge length of the square buffer, in pixels.
const SIZE: u32 = 32;
/// Radius of the red disc, in pixels.
const RADIUS: f32 = 11.0;
/// Distance from the top-right corner of the output.
const MARGIN: i32 = 16;
/// Disc colour, `#e01b24`.
const RED: [f32; 3] = [224.0, 27.0, 36.0];

const SPEC: OverlaySpec = OverlaySpec {
    width: SIZE,
    height: SIZE,
    anchor: Anchor {
        top: true,
        right: true,
        bottom: false,
        left: false,
    },
    margin: MARGIN,
};

/// Shows and hides the indicator. The pixels are drawn once, so each `show`
/// only attaches the existing buffer.
pub struct RecordingIndicator {
    wl: Wayland,
    buffer: ShmBuffer,
    shown: Option<OverlaySurface>,
}

impl RecordingIndicator {
    /// Fails if the compositor cannot provide overlay surfaces.
    pub fn new(wl: &Wayland) -> Result<Self> {
        let buffer = ShmBuffer::new(wl, SIZE, SIZE, &disc_pixels(SIZE, RADIUS))?;
        Ok(Self {
            wl: wl.clone(),
            buffer,
            shown: None,
        })
    }

    /// Shows the indicator. Does nothing if it is already shown.
    pub fn show(&mut self) -> Result<()> {
        if self.shown.is_none() {
            self.shown = Some(OverlaySurface::show(&self.wl, &SPEC, &self.buffer)?);
        }
        Ok(())
    }

    /// Hides the indicator. Does nothing if it is not shown.
    pub fn hide(&mut self) -> Result<()> {
        match self.shown.take() {
            Some(overlay) => overlay.hide(),
            None => Ok(()),
        }
    }
}

/// Draws the disc as premultiplied ARGB8888 (little-endian bytes B, G, R, A).
/// The edge is anti-aliased over one pixel.
fn disc_pixels(size: u32, radius: f32) -> Vec<u8> {
    let centre = size as f32 / 2.0;
    let mut pixels = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - centre;
            let dy = y as f32 + 0.5 - centre;
            let distance = (dx * dx + dy * dy).sqrt();
            let alpha = (radius - distance + 0.5).clamp(0.0, 1.0);
            pixels.push((RED[2] * alpha) as u8);
            pixels.push((RED[1] * alpha) as u8);
            pixels.push((RED[0] * alpha) as u8);
            pixels.push((255.0 * alpha) as u8);
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::{RADIUS, SIZE, disc_pixels};

    fn alpha_at(pixels: &[u8], x: u32, y: u32) -> u8 {
        pixels[((y * SIZE + x) * 4 + 3) as usize]
    }

    #[test]
    fn disc_is_opaque_at_centre_and_clear_in_corners() {
        let pixels = disc_pixels(SIZE, RADIUS);
        assert_eq!(pixels.len(), (SIZE * SIZE * 4) as usize);
        assert_eq!(alpha_at(&pixels, SIZE / 2, SIZE / 2), 255);
        assert_eq!(alpha_at(&pixels, 0, 0), 0);
        assert_eq!(alpha_at(&pixels, SIZE - 1, SIZE - 1), 0);
    }

    #[test]
    fn disc_colour_is_premultiplied_red() {
        let pixels = disc_pixels(SIZE, RADIUS);
        let i = ((SIZE / 2 * SIZE + SIZE / 2) * 4) as usize;
        // Little-endian ARGB8888 is stored as B, G, R, A.
        assert_eq!(&pixels[i..i + 4], &[0x24, 0x1b, 0xe0, 0xff]);
    }

    /// Shows and hides the indicator on a real compositor. Use a private
    /// headless compositor, not a live session.
    #[test]
    #[ignore = "needs a Wayland compositor with zwlr_layer_shell_v1"]
    fn shows_and_hides() {
        let wl = crate::wayland::Wayland::connect().unwrap();
        let mut indicator = super::RecordingIndicator::new(&wl).unwrap();
        indicator.show().unwrap();
        indicator.show().unwrap();
        indicator.hide().unwrap();
        indicator.hide().unwrap();
    }
}
