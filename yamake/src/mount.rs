// use crate::error as E;
use colored_text::Colorize;
use log;
use petgraph::Direction::Incoming;
use petgraph::dot::Dot;
use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::mpsc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::result::Result;
// use std::time::Duration;

use crate::model::MakeReturn;
use crate::target_hash::{compute_needs_rebuild, write_current_hash};
use crate::{model as M, target_hash};
// use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub(crate) fn mount(g: &M::G) -> Result<u32, Box<dyn std::error::Error>> {
    log::info!("mount");
    let mount_text: String = "mount".hex("#444444").italic().underline().bold();
    let pb = ProgressBar::new(g.g.node_indices().count().try_into().unwrap());

    std::fs::create_dir_all(&g.sandbox)?;
    let mut count = 0;

    for id in g.g.node_indices() {
        let _n = g.g.node_weight(id).ok_or("huh ?")?;
        if !g.is_root_node(id) {
            continue;
        }
        {
            log::info!("mount {id:?}");
            let n = g.g.node_weight(id).ok_or("huh ?")?;
            // log::info!("mount {:?}", n.target());

            let mut target_in_srcdir = g.srcdir.clone();
            target_in_srcdir.push(n.target());
            if !target_in_srcdir.exists() {
                // let msg = format!(
                //     r###"""
                // this target node has no predecessor : {}
                // either :
                // - it is a source file that does not exist, check typos or create it
                // - it is a built file, add a link between this node and its predecessors
                // """###,
                //     n.target().display().hex("#FF1493").on_hex("#F0FFFF").bold(),
                // );
                log::error!("target_in_srcdir {:?} does not exist", target_in_srcdir);
                // return Err(E::CouldNotMountFileError::new(n.target()).into());
            }
            let mut target_in_sandbox = g.sandbox.clone();
            target_in_sandbox.push(n.target());

            log::info!("MOUNT {target_in_srcdir:?} => {target_in_sandbox:?}");
            std::fs::create_dir_all(target_in_sandbox.parent().ok_or("no parent ?")?)?;
            std::fs::copy(
                target_in_srcdir.clone().as_os_str(),
                target_in_sandbox.as_os_str(),
            )?;
            pb.println(format!(
                "{mount_text} {:?} ",
                // id_text(ni),
                &n.target(),
            ));
            pb.inc(1);
            count += 1;
        }
    }

    Ok(count)
}
