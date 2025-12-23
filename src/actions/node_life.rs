use crate::model as M;
use petgraph::graph::NodeIndex;
use std::result::Result;
use std::sync::Arc;

pub fn is_root_node(g: &M::G, ni: NodeIndex) -> bool {
    for npred in g.g.edges_directed(ni, petgraph::Incoming) {
        match npred.weight().kind {
            M::EKind::Scanned => (),
            M::EKind::Explicit => {
                return false;
            }
        }
    }
    true
}

fn mount_node(g: &M::G, n: &Arc<dyn M::GNode>) -> Result<bool, Box<dyn std::error::Error>> {
    let mut target_in_srcdir = g.srcdir.clone();
    target_in_srcdir.push(n.target());
    if !target_in_srcdir.exists() {
        return Ok(false);
    }
    let mut target_in_sandbox = g.sandbox.clone();
    target_in_sandbox.push(n.target());

    log::info!("MOUNT {target_in_srcdir:?} => {target_in_sandbox:?}");
    std::fs::create_dir_all(target_in_sandbox.parent().ok_or("no parent ?")?)?;
    std::fs::copy(
        target_in_srcdir.clone().as_os_str(),
        target_in_sandbox.as_os_str(),
    )?;

    Ok(true)
}

pub async fn compute_node_state(
    g: &mut M::G,
    node_idx: NodeIndex,
) -> Result<M::ENodeStatus, Box<dyn std::error::Error>> {
    let node = g.g.node_weight(node_idx).unwrap();
    log::info!("Node {} with tag {}", node.target().display(), node.tag());

    let is_root = is_root_node(g, node_idx);
    log::info!("is_root_node: {}", is_root);

    if is_root {
        match mount_node(g, node)? {
            true => {
                log::info!("Node {} is mounted", node.target().display());
                return Ok(M::ENodeStatus::Changed);
            }
            false => {
                log::info!("Node {} is source", node.target().display());
                return Ok(M::ENodeStatus::MissingSource);
            }
        }
    }

    
    

    log::error!("Not implemented yet for non-root nodes");
    unimplemented!();
}

pub async fn _compute_graph_state(g: &mut M::G) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
