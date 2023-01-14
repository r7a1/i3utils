use anyhow::Result;
use xcb::{x, XidNew};

pub struct Connection(xcb::Connection);

impl Connection {
    pub fn new() -> Result<Self> {
        let (conn, _) = xcb::Connection::connect(None)?;
        Ok(Connection(conn))
    }

    pub fn map(&self, window: u32) -> Result<()> {
        let cookie = self.0.send_request_checked(&x::MapWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.0.check_request(cookie)?)
    }

    pub fn unmap(&self, window: u32) -> Result<()> {
        let cookie = self.0.send_request_checked(&x::UnmapWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.0.check_request(cookie)?)
    }

    pub fn destroy(&self, window: u32) -> Result<()> {
        let cookie = self.0.send_request_checked(&x::DestroyWindow {
            window: unsafe { x::Window::new(window) },
        });
        Ok(self.0.check_request(cookie)?)
    }

    pub fn flush(&self) -> Result<()> {
        Ok(self.0.flush()?)
    }

    pub fn get_pid(&self, window: u32) -> Option<u32> {
        self.intern_atom(b"_NET_WM_PID").and_then(|atom| {
            let cookie = self.0.send_request(&x::GetProperty {
                delete: false,
                window: unsafe { x::Window::new(window) },
                property: atom,
                r#type: x::ATOM_CARDINAL,
                long_offset: 0,
                long_length: 4,
            });
            match self.0.wait_for_reply(cookie) {
                Ok(r) => Some(r.value()[0]),
                Err(_) => None,
            }
        })
    }

    fn intern_atom(&self, name: &[u8]) -> Option<x::Atom> {
        let cookie = self.0.send_request(&x::InternAtom {
            only_if_exists: false,
            name,
        });
        self.0.wait_for_reply(cookie).ok().map(|r| r.atom())
    }
}
