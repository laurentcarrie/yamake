use petgraph::graph::NodeIndex;
use std::path::PathBuf;

pub fn logstream(
    sandArc: &PathBuf,
    id: NodeIndex,
    s: &str,
) -> Result<std::fs::File, Arc<dyn std::error::Error + 'static>> {
    let mut logpath = sandArc.clone();
    logpath.push("log");
    logpath.push(format!("{}-{}.log", id.index(), s));
    Ok(std::fs::File::create(logpath)?)
}
