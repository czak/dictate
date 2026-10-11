use std::fs::File;
use std::io::Write;
use std::os::fd::AsFd;

use anyhow::{Result, ensure};
use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::protocol::{wl_buffer, wl_shm};

use super::Wayland;

/// A pixel buffer in shared memory, in premultiplied ARGB8888. It is written
/// once and can be attached to any number of surfaces.
pub struct ShmBuffer {
    buffer: wl_buffer::WlBuffer,
}

impl ShmBuffer {
    /// Creates a `width` by `height` buffer holding `argb`, which must be
    /// `width * height * 4` bytes of premultiplied ARGB8888 in little-endian
    /// order (B, G, R, A per pixel).
    pub fn new(wl: &Wayland, width: u32, height: u32, argb: &[u8]) -> Result<Self> {
        let len = (width as usize) * (height as usize) * 4;
        ensure!(
            argb.len() == len,
            "pixel data is {} bytes, expected {len}",
            argb.len()
        );

        let fd = memfd_create("dictate-shm", MemfdFlags::CLOEXEC)?;
        let mut file = File::from(fd);
        file.write_all(argb)?;

        let shm: wl_shm::WlShm = wl.shm()?;
        let pool = shm.create_pool(file.as_fd(), len as i32, &wl.qh(), ());
        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            (width * 4) as i32,
            wl_shm::Format::Argb8888,
            &wl.qh(),
            (),
        );
        pool.destroy();
        // The file must stay open until the compositor has received the fd.
        wl.roundtrip()?;
        drop(file);
        Ok(Self { buffer })
    }

    pub(super) fn proxy(&self) -> &wl_buffer::WlBuffer {
        &self.buffer
    }
}
