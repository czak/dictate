//! Wayland client code. The rest of the daemon uses the types re-exported here
//! and never touches protocol objects directly.
//!
//! One [`Wayland`] connection serves the whole daemon. It binds the globals the
//! daemon needs and dispatches events for all of them.

mod overlay;
mod shm;
mod virtual_keyboard;

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use wayland_client::protocol::{
    wl_buffer, wl_compositor, wl_region, wl_registry, wl_seat, wl_shm, wl_shm_pool, wl_surface,
};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, delegate_noop};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1;
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1;

pub use overlay::{Anchor, OverlaySpec, OverlaySurface};
pub use shm::ShmBuffer;
pub use virtual_keyboard::VirtualKeyboard;

/// A connection to the Wayland display. Cloning it gives another handle to the
/// same connection. The daemon is single-threaded, so the handles share it with
/// `Rc`.
#[derive(Clone)]
pub struct Wayland(Rc<RefCell<Inner>>);

struct Inner {
    /// Kept alive for as long as the queue and the proxies are in use.
    _conn: Connection,
    queue: EventQueue<State>,
    state: State,
}

/// The globals bound from the registry. Each one is `None` if the compositor
/// does not advertise it.
#[derive(Default)]
pub(super) struct State {
    seat: Option<wl_seat::WlSeat>,
    virtual_keyboard_manager: Option<ZwpVirtualKeyboardManagerV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    shm: Option<wl_shm::WlShm>,
    layer_shell: Option<ZwlrLayerShellV1>,
}

impl Wayland {
    /// Connects to the display named by the environment and binds the globals.
    pub fn connect() -> Result<Self> {
        let conn = Connection::connect_to_env().context("cannot connect to Wayland display")?;
        let mut queue = conn.new_event_queue();
        let qh = queue.handle();
        let _registry = conn.display().get_registry(&qh, ());
        let mut state = State::default();
        queue.roundtrip(&mut state)?;
        Ok(Self(Rc::new(RefCell::new(Inner {
            _conn: conn,
            queue,
            state,
        }))))
    }

    pub(super) fn qh(&self) -> QueueHandle<State> {
        self.0.borrow().queue.handle()
    }

    /// Sends pending requests and waits until the compositor has handled them.
    /// Events it sent in the meantime are dispatched.
    pub(super) fn roundtrip(&self) -> Result<()> {
        let mut inner = self.0.borrow_mut();
        let Inner { queue, state, .. } = &mut *inner;
        queue.roundtrip(state)?;
        Ok(())
    }

    /// Dispatches events until `done` returns true.
    pub(super) fn wait_until(&self, done: impl Fn() -> bool) -> Result<()> {
        let mut inner = self.0.borrow_mut();
        let Inner { queue, state, .. } = &mut *inner;
        while !done() {
            queue.blocking_dispatch(state)?;
        }
        Ok(())
    }

    pub(super) fn seat(&self) -> Result<wl_seat::WlSeat> {
        self.global(|s| s.seat.clone(), "wl_seat")
    }

    pub(super) fn virtual_keyboard_manager(&self) -> Result<ZwpVirtualKeyboardManagerV1> {
        self.global(
            |s| s.virtual_keyboard_manager.clone(),
            "zwp_virtual_keyboard_manager_v1",
        )
    }

    pub(super) fn compositor(&self) -> Result<wl_compositor::WlCompositor> {
        self.global(|s| s.compositor.clone(), "wl_compositor")
    }

    pub(super) fn shm(&self) -> Result<wl_shm::WlShm> {
        self.global(|s| s.shm.clone(), "wl_shm")
    }

    pub(super) fn layer_shell(&self) -> Result<ZwlrLayerShellV1> {
        self.global(|s| s.layer_shell.clone(), "zwlr_layer_shell_v1")
    }

    fn global<T>(&self, get: impl Fn(&State) -> Option<T>, interface: &str) -> Result<T> {
        let inner = self.0.borrow();
        get(&inner.state).with_context(|| format!("compositor does not advertise {interface}"))
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_seat" if state.seat.is_none() => {
                    let version = version.min(wl_seat::WlSeat::interface().version);
                    state.seat = Some(registry.bind(name, version, qh, ()));
                }
                "wl_compositor" => {
                    let version = version.min(wl_compositor::WlCompositor::interface().version);
                    state.compositor = Some(registry.bind(name, version, qh, ()));
                }
                "wl_shm" => {
                    state.shm = Some(registry.bind(name, 1, qh, ()));
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    state.virtual_keyboard_manager = Some(registry.bind(name, 1, qh, ()));
                }
                "zwlr_layer_shell_v1" => {
                    state.layer_shell = Some(registry.bind(name, 1, qh, ()));
                }
                _ => {}
            }
        }
    }
}

delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_noop!(State: ignore wl_compositor::WlCompositor);
delegate_noop!(State: ignore wl_shm::WlShm);
delegate_noop!(State: ignore wl_shm_pool::WlShmPool);
delegate_noop!(State: ignore wl_buffer::WlBuffer);
delegate_noop!(State: ignore wl_surface::WlSurface);
delegate_noop!(State: ignore wl_region::WlRegion);
delegate_noop!(State: ignore ZwpVirtualKeyboardManagerV1);
delegate_noop!(State: ignore ZwpVirtualKeyboardV1);
delegate_noop!(State: ignore ZwlrLayerShellV1);
