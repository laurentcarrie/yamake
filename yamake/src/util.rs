use petgraph::graph::NodeIndex;
use std::path::PathBuf;

pub fn logstream(
    sandbox: &PathBuf,
    id: NodeIndex,
    s: &str,
) -> Result<std::fs::File, Box<dyn std::error::Error + 'static>> {
    let mut logpath = sandbox.clone();
    logpath.push("log");
    logpath.push(format!("{}-{}.log", id.index(), s));
    Ok(std::fs::File::create(logpath)?)
}
