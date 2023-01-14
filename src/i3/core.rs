use anyhow::Result;
use i3ipc::I3Connection;

pub struct Core {
    conn: I3Connection,
}

impl Core {
    pub fn new() -> Result<Core> {
        Ok(Core {
            conn: I3Connection::connect()?,
        })
    }

    pub fn run(&mut self, cmd: &str) -> Result<()> {
        self.conn.run_command(cmd)?;
        Ok(())
    }

    pub fn run_batch(&mut self, cmds: BatchBuilder) -> Result<()> {
        dbg!(&cmds);
        self.conn.run_command(&cmds.build())?;
        Ok(())
    }

    // All i3 nodes have `con_id`
    pub fn focus(&mut self, id: i64) -> Result<()> {
        self.run(&format!(r#"[con_id="{id}"] focus"#))?;
        Ok(())
    }

    pub fn focus_window(&mut self, id: i32) -> Result<()> {
        self.run(&format!(r#"[id="{id}"] focus"#))?;
        Ok(())
    }

    pub fn get_tree(&mut self) -> Result<i3ipc::reply::Node> {
        Ok(self.conn.get_tree()?)
    }
}

#[derive(Debug)]
pub struct BatchBuilder {
    cmds: Vec<String>,
}

impl BatchBuilder {
    pub fn new() -> BatchBuilder {
        BatchBuilder { cmds: vec![] }
    }

    pub fn push(&mut self, cmd: &str) {
        self.cmds.push(cmd.to_string());
    }

    fn build(&self) -> String {
        self.cmds.join(";")
    }
}
