use anyhow::Result;

pub struct Connection(xcb::Connection);

impl Connection {
    pub fn new() -> Result<Self> {
        Ok(Connection(xcb::Connection::connect(None)?.0))
    }

    pub fn map(&self, window: u32) -> Result<()> {
        Ok(xcb::xproto::map_window(&self.0, window).request_check()?)
    }

    pub fn unmap(&self, window: u32) -> Result<()> {
        Ok(xcb::xproto::unmap_window(&self.0, window).request_check()?)
    }

    pub fn destroy(&self, window: u32) -> Result<()> {
        Ok(xcb::xproto::destroy_window(&self.0, window).request_check()?)
    }

    pub fn flush(&self) {
        self.0.flush();
    }

    pub fn get_pid(&self, window: u32) -> Option<u32> {
        self.intern_atom("_NET_WM_PID").and_then(|atom| {
            match xcb::xproto::get_property(&self.0, false, window, atom, xcb::ATOM_CARDINAL, 0, 4)
                .get_reply()
            {
                Ok(r) => Some(r.value()[0]),
                Err(_) => None,
            }
        })
    }

    fn intern_atom(&self, name: &str) -> Option<xcb::Atom> {
        xcb::xproto::intern_atom(&self.0, true, name)
            .get_reply()
            .ok()
            .map(|r| r.atom())
    }
}
