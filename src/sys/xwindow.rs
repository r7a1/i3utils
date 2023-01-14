use anyhow::Result;
use xcb::{x, XidNew};

pub struct Connection {
    inner: xcb::Connection,
}

impl Connection {
    pub fn new() -> Result<Self> {
        let (conn, _) = xcb::Connection::connect(None)?;
        Ok(Connection { inner: conn })
    }

    /// Maps a X window.
    pub fn map(&self, window: u32) -> Result<()> {
        let cookie = self.inner.send_request_checked(&x::MapWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.inner.check_request(cookie)?)
    }

    /// Unmaps a X window.
    pub fn unmap(&self, window: u32) -> Result<()> {
        let cookie = self.inner.send_request_checked(&x::UnmapWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.inner.check_request(cookie)?)
    }

    /// Destroys a X window.
    pub fn destroy(&self, window: u32) -> Result<()> {
        let cookie = self.inner.send_request_checked(&x::DestroyWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.inner.check_request(cookie)?)
    }

    /// Forces buffered output to the X server.
    pub fn flush(&self) -> Result<()> {
        Ok(self.inner.flush()?)
    }

    /// Retrieve PID associated to a window.
    pub fn get_pid(&self, window: u32) -> Option<u32> {
        self.intern_atom(b"_NET_WM_PID").and_then(|atom| {
            let cookie = self.inner.send_request(&x::GetProperty {
                delete: false,
                window: unsafe { x::Window::new(window) },
                property: atom,
                r#type: x::ATOM_CARDINAL,
                long_offset: 0,
                long_length: 4,
            });
            match self.inner.wait_for_reply(cookie) {
                Ok(r) => Some(r.value()[0]),
                Err(_) => None,
            }
        })
    }

    /// Retrieves the identifier for the atom with a specified name.
    fn intern_atom(&self, name: &[u8]) -> Option<x::Atom> {
        let cookie = self.inner.send_request(&x::InternAtom {
            only_if_exists: false,
            name,
        });
        self.inner.wait_for_reply(cookie).ok().map(|r| r.atom())
    }
}
