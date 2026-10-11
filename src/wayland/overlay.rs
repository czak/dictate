//! A layer-shell surface that floats above windows.
//!
//! The surface is on the overlay layer and never takes keyboard focus. Its input
//! region is empty, so pointer events pass through it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use wayland_client::protocol::wl_surface;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::{
    self, Anchor as WlrAnchor, KeyboardInteractivity, ZwlrLayerSurfaceV1,
};

use super::{ShmBuffer, State, Wayland};

/// Namespace reported to the compositor, which it may use for rules.
const NAMESPACE: &str = "dictate";

/// The screen edges a surface is pinned to. A surface pinned to opposite edges
/// is centred along that axis.
#[derive(Debug, Clone, Copy, Default)]
pub struct Anchor {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

/// Where and how large an overlay is.
#[derive(Debug, Clone, Copy)]
pub struct OverlaySpec {
    pub width: u32,
    pub height: u32,
    pub anchor: Anchor,
    /// Distance in pixels from each anchored edge.
    pub margin: i32,
}

/// A visible overlay. Dropping it removes the overlay, but only [`hide`] waits
/// for the compositor to process the removal.
///
/// [`hide`]: OverlaySurface::hide
pub struct OverlaySurface {
    wl: Wayland,
    shown: Option<Shown>,
}

struct Shown {
    surface: wl_surface::WlSurface,
    layer: ZwlrLayerSurfaceV1,
}

impl OverlaySurface {
    /// Shows `buffer` as an overlay described by `spec`. Blocks until the
    /// compositor has configured the surface.
    pub fn show(wl: &Wayland, spec: &OverlaySpec, buffer: &ShmBuffer) -> Result<Self> {
        let qh = wl.qh();
        let surface = wl.compositor()?.create_surface(&qh, ());
        let configured = Arc::new(AtomicBool::new(false));
        let layer = wl.layer_shell()?.get_layer_surface(
            &surface,
            None,
            Layer::Overlay,
            NAMESPACE.to_string(),
            &qh,
            Arc::clone(&configured),
        );
        // Dropping `this` on an error below destroys the surface.
        let this = Self {
            wl: wl.clone(),
            shown: Some(Shown {
                surface: surface.clone(),
                layer: layer.clone(),
            }),
        };

        layer.set_size(spec.width, spec.height);
        layer.set_anchor(anchor_flags(spec.anchor));
        layer.set_margin(spec.margin, spec.margin, spec.margin, spec.margin);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);

        let region = wl.compositor()?.create_region(&qh, ());
        surface.set_input_region(Some(&region));
        region.destroy();
        surface.commit();

        wl.wait_until(|| configured.load(Ordering::SeqCst))?;

        surface.attach(Some(buffer.proxy()), 0, 0);
        surface.damage_buffer(0, 0, spec.width as i32, spec.height as i32);
        surface.commit();
        wl.roundtrip()?;
        Ok(this)
    }

    /// Removes the overlay and waits until the compositor has processed it.
    pub fn hide(mut self) -> Result<()> {
        self.destroy();
        self.wl.roundtrip()
    }

    fn destroy(&mut self) {
        if let Some(shown) = self.shown.take() {
            shown.layer.destroy();
            shown.surface.destroy();
        }
    }
}

impl Drop for OverlaySurface {
    fn drop(&mut self) {
        self.destroy();
    }
}

fn anchor_flags(anchor: Anchor) -> WlrAnchor {
    let mut flags = WlrAnchor::empty();
    flags.set(WlrAnchor::Top, anchor.top);
    flags.set(WlrAnchor::Right, anchor.right);
    flags.set(WlrAnchor::Bottom, anchor.bottom);
    flags.set(WlrAnchor::Left, anchor.left);
    flags
}

impl Dispatch<ZwlrLayerSurfaceV1, Arc<AtomicBool>> for State {
    fn event(
        _: &mut Self,
        surface: &ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        configured: &Arc<AtomicBool>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_layer_surface_v1::Event::Configure { serial, .. } = event {
            surface.ack_configure(serial);
            configured.store(true, Ordering::SeqCst);
        }
    }
}
