// use crate::error as E;
use colored_text::Colorize;
use log;
use petgraph::Direction::Incoming;
// use petgraph::dot::Dot;
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::result::Result;
// use std::time::Duration;

use crate::model as M;
use crate::model::PathWithTag;
// use tokio::sync::mpsc::Receiver;

pub async fn scan(g: &mut M::G) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("scan");
    let nb_of_egdes_before = g.g.edge_count();

    let _id_text = |id: NodeIndex| -> String {
        format!("{:3}", id.index())
            .hex("#8B008B")
            .on_hex("#FFFF7F")
            .bold()
    };
    let pb = ProgressBar::new(g.g.node_indices().count().try_into().unwrap());
    pb.set_style(
        ProgressStyle::with_template(
            "SCAN  [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    let mut logpath = g.sandbox.clone();
    logpath.push("log");

    // we only scan nodes that are not sources
    let mut nodes_to_scan: Vec<(NodeIndex, &Arc<dyn M::GNode>)> = Vec::new();
    for ni in g.g.node_indices() {
        if g.g.edges_directed(ni, Incoming).count() as u32 > 0 {
            let node = &g.g.node_weight(ni).ok_or("huh ?")?;

            nodes_to_scan.push((ni, node));
        }
    }
    let mut edges_to_add: Vec<(NodeIndex, NodeIndex)> = Vec::new();

    for (ni, n) in nodes_to_scan {
        let mut preds: Vec<PathWithTag> = vec![];
        for ni2 in g.g.neighbors_directed(ni, Incoming) {
            let n2 = g.g.node_weight(ni2).ok_or("hugh ?")?;
            preds.push(M::PathWithTag {
                path: n2.target(),
                tag: n2.tag(),
            })
        }
        let scanned_deps = &n.scan(g.sandbox.clone(), preds)?;
        log::info!(
            "found {} deps for node {:?}",
            scanned_deps.len(),
            n.target()
        );
        for p in scanned_deps {
            // let p = p.strip_prefix(g.srcdir)?.to_path_buf();
            match &g.ni_of_path(p.clone()) {
                Ok(ni_dep) => {
                    log::info!("add scanned edge {ni_dep:?} => {ni:?}");
                    edges_to_add.push((*ni_dep, ni));
                }
                Err(_) => {
                    log::warn!("could not resolve dep {p:?}");
                    // if a scanned dependency does not exist, then it will not be copied to the sandbox and the build will fail
                }
            }
        }

        let scan_text: String = "scanned".hex("#444444").italic().underline().bold();
        let tag_text = |tag: String| tag.hex("#000033").on_hex("#eeeeee").bold();

        pb.println(format!(
            "{scan_text} {:?} [{}] : added {} scanned edge(s)",
            // id_text(ni),
            &n.target(),
            tag_text(n.tag()),
            scanned_deps.len()
        ));
        pb.inc(1);
    }

    for (a, b) in edges_to_add {
        g.g.try_add_edge(
            a,
            b,
            M::E {
                kind: M::EKind::Scanned,
            },
        )?;
    }
    let nb_of_egdes_after = g.g.edge_count();

    Ok(nb_of_egdes_after > nb_of_egdes_before)
}
