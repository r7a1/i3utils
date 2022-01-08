use anyhow::Result;
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::info;
use std::fs::File;
use std::io::{prelude::*, BufReader, Write};
use std::ops::Index;
use std::path::Path;
use std::{
    convert::{TryFrom, TryInto},
    fs,
};

use crate::sys::xwindow;

mod core;

lazy_static! {
    static ref CACHE_DIR: String = ProjectDirs::from("", "", "i3ctl")
        .unwrap()
        .cache_dir()
        .to_str()
        .unwrap()
        .to_string();
    static ref LAYOUT_FILE: String = format!("{}/workspace_1_layout.json", &*CACHE_DIR);
    static ref FOCUS_WID_FILE: String = format!("{}/focused.json", &*CACHE_DIR);
}

pub struct Util(core::Core);

impl Util {
    pub fn new() -> Result<Util> {
        Ok(Util(core::Core::new()?))
    }

    pub fn toggle_fullscreen(&mut self) -> Result<()> {
        if self.has_layout_backup() {
            self.restore_layout()?;
        } else {
            self.save_layout()?;
            if let Err(e) = self.fullscreen() {
                dbg!(e);
                self.restore_layout()?;
            }
        }
        Ok(())
    }

    pub fn clean_layout_backup(&self) -> std::io::Result<()> {
        fs::remove_dir_all(&*CACHE_DIR)
    }

    pub fn focus_nextmatch(&mut self, re: String) -> Result<()> {
        // If `re` is not supecified, use CLASS property of the window.
        // Empty string means no filter.
        let pat = match re {
            _ if re.is_empty() => {
                if let Some(Window { class: Some(c), .. }) = self.focused_window()? {
                    c
                } else {
                    // TODO: Should be handled as an error?
                    return Ok(());
                }
            }
            re => re,
        };
        let pat = regex::Regex::new(&pat)?;

        let mut windows = search_windows(&self.0.get_tree()?, &pat)?.collect::<Vec<_>>();

        if windows.is_empty() {
            return Ok(());
        }

        windows.sort_by_key(|w| w.id);
        // If any window is focused, choose next window. Focus one otherwise.
        if let Some(idx) = windows.iter().position(|w| w.focused) {
            self.0.focus(windows[(idx + 1) % windows.len()].id)
        } else {
            self.0.focus(windows[0].id)
        }
    }

    pub fn run_or_raise(&mut self, cmd: &str, class: &str) -> Result<()> {
        let tree = self.0.get_tree()?;
        let pat = regex::Regex::new(class)?;

        // If matched windows found, focus next. Run command otherwise.

        if search_windows(&tree, &pat)?.next().is_none() {
            info!("run command: {}", cmd);
            // Deamonize process to run command in background.
            duct::cmd!("daemonize", cmd).run()?;
        } else {
            self.focus_nextmatch(class.to_owned())?;
        }
        Ok(())
    }

    fn focused_window(&mut self) -> Result<Option<Window>> {
        Ok(self
            .0
            .get_tree()?
            .focused_node()
            .and_then(|(_, n)| n.try_into().ok()))
    }

    fn has_layout_backup(&self) -> bool {
        info!("layout backup found");
        Path::new(&*LAYOUT_FILE).exists()
    }

    fn restore_layout(&mut self) -> Result<()> {
        info!("restoring layout");
        self.restore_workspace("1", None as Option<&Path>)?;

        if let Ok(file) = File::open(&*FOCUS_WID_FILE) {
            if let Some(line) = BufReader::new(file).lines().next() {
                self.0.focus_window(line?.parse()?)?;
            }
        }
        self.clean_layout_backup()?;
        Ok(())
    }

    pub fn restore_workspace<P>(&mut self, _ws: &str, _dir: Option<P>) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let tree = Util::raw_get_tree()?;
        let xconn = xwindow::Connection::new()?;

        let windows = json_util::traverse(&tree)
            .filter(|n| n.index("window_properties").is_object())
            .filter_map(|n| n.index("window").as_u32())
            .collect::<Vec<_>>();

        for id in &windows {
            xconn.unmap(*id)?;
            if xconn.get_pid(*id).is_none() {
                xconn.destroy(*id)?;
            }
        }
        xconn.flush();

        self.0.run(&format!("append_layout {}", &*LAYOUT_FILE))?;

        // As i3 layout does not remember window order,
        // windows must be swallowed in order.
        for w in windows.iter().rev() {
            xconn.map(*w)?;
        }
        xconn.flush();

        Ok(())
    }

    fn raw_get_tree() -> Result<json::JsonValue> {
        Ok(json::parse(
            &duct::cmd!("i3-msg", "-t", "get_tree").read()?,
        )?)
    }

    fn save_layout(&mut self) -> Result<()> {
        info!("saving current layout");
        if let Some((
            _,
            i3ipc::reply::Node {
                window: Some(id), ..
            },
        )) = self.0.get_tree()?.focused_node()
        {
            fs::create_dir(&*CACHE_DIR)?;
            fs::File::create(&*FOCUS_WID_FILE)?.write_all(id.to_string().as_bytes())?;
        }

        Util::save_workspace("1")?;

        Ok(())
    }

    pub fn save_workspace(workspace: &str) -> Result<()> {
        if let Some(ws) = Util::raw_get_tree()?
            .index("nodes")
            .members()
            .flat_map(|n| n.index("nodes").members())
            .flat_map(|n| n.index("nodes").members())
            .find(|n| n.index("name").as_str() == Some(workspace))
            .map(|n| n.index("nodes"))
        {
            let mut f = File::create(&*LAYOUT_FILE)?;

            f.write_all(Util::build_tree(ws)?.to_string().as_bytes())?;
        }
        Ok(())
    }

    fn build_tree(src: &json::JsonValue) -> Result<json::JsonValue> {
        let mut dst = json::JsonValue::new_array();

        if src.is_empty() {
            return Ok(dst);
        }

        // Dig nodes recursively.
        for node in src.members() {
            let mut container = json::JsonValue::new_object();

            // Copy specific items.
            for key in &[
                "border",
                "current_border_width",
                "floating",
                "fullscreen_mode",
                "geometry",
                "layout",
                "name",
                "orientation",
                "percent",
                "scratchpad_state",
                "type",
                "workspace_layout",
            ] {
                if node.has_key(&key) {
                    container.insert(key, node.index(*key).clone())?;
                }
            }

            // Set `swallows`.
            if node.has_key("window_properties") {
                let mut swallows = json::JsonValue::new_object();
                for key in &["class", "instance"] {
                    if let Some(val) = node.index("window_properties").index(*key).as_str() {
                        swallows.insert(key, format!("^{}$", val))?;
                    }
                }
                container.insert("swallows", swallows)?;
            }

            // Process children.
            container.insert("nodes", Util::build_tree(node.index("nodes"))?)?;

            dst.push(container)?;
        }

        Ok(dst)
    }

    /// Emulate fullscreen by applying tab-layout recursively, achieving
    /// menu-ber leaving fullscreen behavior.
    fn fullscreen(&mut self) -> Result<()> {
        let tree = self.0.get_tree()?;
        let mut cmds = core::BatchBuilder::new();

        if let Some((_, i3ipc::reply::Node { id, .. })) = tree.focused_node() {
            // Apply tab layout to all layouts of at bottom to top.
            for _ in tree
                .focused_nodes()
                .enumerate()
                .skip_while(|(d, n)| (n.is_window() && d <= &3) || (n.is_container() && d <= &4))
                .take_while(|(_, n)| !n.focused)
            {
                cmds.push("layout tabbed");
                cmds.push("focus parent");
            }
            cmds.push("layout tabbed");
            cmds.push(&format!(r#"[con_id="{}"] focus"#, *id));
            self.0.run_batch(cmds)?;
        }
        Ok(())
    }
}

trait Node {
    fn is_window(&self) -> bool;
    fn is_container(&self) -> bool;
    fn get_windows<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Window> + 'a>>;
    fn traverse<'a>(&'a self) -> Nodes<'a>;
    fn focused_node(&self) -> Option<(usize, &Self)>;
    fn focused_nodes<'a>(&'a self) -> FocusedNodes<'a>;
}

impl Node for i3ipc::reply::Node {
    fn is_window(&self) -> bool {
        self.window.is_some()
    }

    fn is_container(&self) -> bool {
        !self.is_window() && self.nodetype == i3ipc::reply::NodeType::Con
    }

    fn get_windows<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Window> + 'a>> {
        Ok(Box::new(
            self.traverse()
                .filter(|node| node.geometry.2 != 0)
                .filter(|node| node.nodetype == i3ipc::reply::NodeType::Con)
                .filter(|node| node.name.is_some())
                .filter_map(|node| node.try_into().ok()),
        ))
    }

    fn traverse<'a>(&'a self) -> Nodes<'a> {
        Nodes { nodes: vec![self] }
    }

    fn focused_node(&self) -> Option<(usize, &Self)> {
        self.focused_nodes()
            .enumerate()
            .skip_while(|(_, n)| !n.focused)
            .next()
    }

    fn focused_nodes<'a>(&'a self) -> FocusedNodes<'a> {
        FocusedNodes { next: Some(self) }
    }
}

struct Nodes<'a> {
    nodes: Vec<&'a i3ipc::reply::Node>,
}

impl<'a> Iterator for Nodes<'a> {
    type Item = &'a i3ipc::reply::Node;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.nodes.pop();
        if let Some(node) = next {
            // Drop dock windows.
            if node.layout != i3ipc::reply::NodeLayout::DockArea {
                self.nodes.extend(&node.nodes);
            }
        }
        next
    }
}

struct FocusedNodes<'a> {
    next: Option<&'a i3ipc::reply::Node>,
}

impl<'a> Iterator for FocusedNodes<'a> {
    type Item = &'a i3ipc::reply::Node;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next;
        if let Some(node) = current {
            self.next = if !node.focus.is_empty() {
                node.nodes.iter().find(|n| n.id == node.focus[0])
            } else {
                None
            };
        }
        current
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Window {
    pub id: i64,
    pub name: Option<String>,
    pub class: Option<String>,
    pub focused: bool,
}

impl TryFrom<&i3ipc::reply::Node> for Window {
    type Error = anyhow::Error;

    fn try_from(node: &i3ipc::reply::Node) -> Result<Self> {
        if !node.is_window() {
            return Err(std::io::Error::from(std::io::ErrorKind::AddrInUse).into());
        }
        Ok(Window {
            id: node.id,
            name: node.name.clone(),
            class: node
                .window_properties
                .as_ref()
                .and_then(|map| map.get(&i3ipc::reply::WindowProperty::Class))
                .map(|class| class.to_string()),
            focused: node.focused,
        })
    }
}

fn search_windows<'a>(
    node: &'a i3ipc::reply::Node,
    name: &'a regex::Regex,
) -> Result<impl Iterator<Item = Window> + 'a> {
    Ok(node.get_windows()?.filter(move |w| {
        w.name.as_ref().and_then(|n| name.find(n)).is_some()
            || w.class.as_ref().and_then(|c| name.find(c)).is_some()
    }))
}

mod json_util {
    pub struct Traverse<'a> {
        nodes: Vec<&'a json::JsonValue>,
    }

    impl<'a> Traverse<'a> {
        fn new(node: &'a json::JsonValue) -> Traverse<'a> {
            Traverse { nodes: vec![node] }
        }
    }

    impl<'a> Iterator for Traverse<'a> {
        type Item = &'a json::JsonValue;

        fn next(&mut self) -> Option<Self::Item> {
            let next = self.nodes.pop();
            if let Some(node) = next {
                self.nodes.extend(node["nodes"].members());
            }
            next
        }
    }

    pub fn traverse<'n>(node: &'n json::JsonValue) -> Traverse<'n> {
        Traverse::new(node)
    }
}

// #[cfg(test)]
// mod tests {
// use super::*;

// #[test]
// fn test_test() {
//     json::parse(&duct::cmd!("i3-msg", "-t", "get_tree").read().unwrap()).unwrap();
// }

// #[test]
// fn test_pid() {
//     let conn = xwindow::Connection::new().unwrap();
//     dbg!(conn.get_pid(35651594));
// }

// #[test]
// fn test_kill() {
//     let conn = xwindow::Connection::new().unwrap();
//     conn.unmap(29360138).unwrap();
//     std::thread::sleep(std::time::Duration::from_secs(5));
//     conn.flush();
// }

// #[test]
// fn test_restore() {
//     let mut ctl = Util::new().unwrap();
//     ctl.restore_workspace("1", None as Option<&Path>).unwrap();
// }

// #[test]
// fn test_workspace() {
//     Util::save_workspace("1", Some(&std::path::Path::new("/tmp"))).unwrap();
// }
// }
