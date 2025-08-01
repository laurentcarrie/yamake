use colored_text::Colorize;
use log;
use petgraph::Direction::Incoming;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::result::Result;
// use std::time::Duration;

use crate::model as M;
// use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub(crate) fn mount(g: &M::G) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("mount");
    std::fs::create_dir_all(&g.sandbox)?;

    for id in g.g.node_indices() {
        let _n = g.g.node_weight(id).ok_or("huh ?")?;
        if !g.is_root_node(id) {
            continue;
        }
        {
            log::info!("mount {id:?}");
            let n = g.g.node_weight(id).ok_or("huh ?")?;
            log::info!("mount {:?}", n.target());

            let mut target_in_srcdir = g.srcdir.clone();
            target_in_srcdir.push(n.target());
            if !target_in_srcdir.exists() {
                let msg = format!(
                    r###"""
                this target node has no predecessor : {}
                either :
                - it is a source file that does not exist, check typos or create it
                - it is a built file, add a link between this node and its predecessors
                """###,
                    n.target().display().hex("#FF1493").on_hex("#F0FFFF").bold(),
                );
                return Err(msg.into());
            }
            let mut target_in_sandbox = g.sandbox.clone();
            target_in_sandbox.push(n.target());

            log::info!("MOUNT {target_in_srcdir:?} => {target_in_sandbox:?}");
            std::fs::create_dir_all(target_in_sandbox.parent().ok_or("no parent ?")?)?;
            std::fs::copy(
                target_in_srcdir.clone().as_os_str(),
                target_in_sandbox.as_os_str(),
            )?;
        }
    }

    Ok(true)
}

pub(crate) async fn make(
    g: &M::G,
    _force_rebuild: bool,
    nb_workers: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    mount(g)?;
    let (tx, mut rx) = mpsc::channel::<(NodeIndex, M::BuildType)>(1000);

    let mut set: JoinSet<()> = JoinSet::new();

    log::info!("{}:{} SCAN", file!(), line!());
    // g.scan().await?;

    // let g: petgraph::Graph<M::N, M::E> = g.g;

    // let done_text = "DONE".hex("#8B008B").on_hex("#7FFF00").bold();
    let built_text = " BUILT ".hex("#7FFF00").bold();
    let _not_touched_text = "Skip".hex("#8B008B").on_hex("#7FFFFF").bold();
    let failed_text = "FAILED"
        .hex("#FF1493")
        // .on_hex("#F0FFFF")
        .on_hex("#d38a8aff")
        .bold();
    // let ancestor_failed_text = "Ancestor Failed".hex("#FF8C00").on_hex("#000000").bold();
    let ancestor_failed_text = "Ancestor Failed".hex("#FF8C00").bold();
    let id_text = |id: NodeIndex| -> String {
        format!("{:3}", id.index())
            .hex("#8B008B")
            .on_hex("#7FFFFF")
            .bold()
    };

    let pb = ProgressBar::new(g.g.node_indices().count().try_into().unwrap());
    pb.set_style(
        ProgressStyle::with_template(
            ":-) make  [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    let mut pending: HashSet<NodeIndex> = HashSet::new();
    for n in g.g.node_indices() {
        pending.insert(n);
    }

    let mut running: HashSet<NodeIndex> = HashSet::new();
    let mut rebuilt: HashSet<NodeIndex> = HashSet::new();
    let mut built_targets: HashMap<NodeIndex, PathBuf> = HashMap::new();
    let mut skipped: HashSet<NodeIndex> = HashSet::new();
    let mut failed: HashSet<NodeIndex> = HashSet::new();
    let mut ancestor_failed: HashSet<NodeIndex> = HashSet::new();

    let total_nodes = g.g.node_indices().count();
    'outermost: loop {
        log::info!(
            "stats : {} pending ; {} running ; {} rebuilt ; {} failed ; {} ancestor failed ;  {} skipped, nb_workers={} ; total_nodes={}",
            pending.len(),
            running.len(),
            rebuilt.len(),
            failed.len(),
            ancestor_failed.len(),
            skipped.len(),
            nb_workers,
            total_nodes
        );
        if pending.len()
            + running.len()
            + rebuilt.len()
            + failed.len()
            + ancestor_failed.len()
            + skipped.len()
            != total_nodes
        {
            log::error!("pending : {pending:?}");
            log::error!("running : {running:?}");
            log::error!("rebuilt : {rebuilt:?}");
            log::error!("failed : {failed:?}");
            log::error!("ancestor_failed : {ancestor_failed:?}");
            log::error!("skipped : {skipped:?}");
        }
        assert!(
            pending.len()
                + running.len()
                + rebuilt.len()
                + failed.len()
                + ancestor_failed.len()
                + skipped.len()
                == total_nodes
        );
        if total_nodes == rebuilt.len() + failed.len() + ancestor_failed.len() + skipped.len() {
            break 'outermost;
        }
        // 'outer: loop {
        {
            // log::info!("running: {:?}", running.len());
            // if running.len() == nb_workers as usize {
            //     log::info!("break 'outer");
            //     break 'outer;
            // }
            if pending.is_empty() {
                break;
            }

            {
                for ni in g.g.node_indices() {
                    if running.len() == nb_workers as usize {
                        log::info!("break 'outer");
                        break;
                    }
                    if !pending.contains(&ni) {
                        continue;
                    }
                    let node = g.g.node_weight(ni).ok_or("huh, no node?")?;
                    let mut ok_to_start = true;
                    let mut an_ancestor_failed = false;
                    let mut an_ancestor_changed = false;

                    for p in g.g.neighbors_directed(ni, petgraph::Direction::Incoming) {
                        if !rebuilt.contains(&p) && !skipped.contains(&p) {
                            ok_to_start = false;
                        }
                        if failed.contains(&p) || ancestor_failed.contains(&p) {
                            an_ancestor_failed = true;
                        }
                        if rebuilt.contains(&p) {
                            an_ancestor_changed = true;
                        }
                    }
                    if an_ancestor_failed {
                        log::info!("ANCESTOR FAILED === > {:?} ; {:?}", node.target(), ni);
                        pending.remove(&ni);
                        ancestor_failed.insert(ni);
                        match tx.send((ni, M::BuildType::AncestorFailed)).await {
                            Ok(()) => {
                                // log::info!("ok, sent");
                                ()
                            }
                            Err(e) => log::error!("failed to send node index: {ni:?} {e}"),
                        };
                    // } else if ok_to_start && !an_ancestor_changed {
                    //     log::info!("SKIP === > {:?}", node);
                    //     pending.remove(&ni);
                    //     skipped.insert(ni);
                    //     match tx
                    //         .send((ni, M::BuildType::NotTouched(PathBuf::from(""))))
                    //         .await
                    //     {
                    //         Ok(()) => {
                    //             // log::info!("ok, sent");
                    //             ()
                    //         }
                    //         Err(e) => log::error!("failed to send node index: {:?} {}", ni, e),
                    //     };
                    } else if ok_to_start {
                        log::info!("START === > node {:?} ; id {:?}", node.target(), ni);
                        pending.remove(&ni);
                        running.insert(ni);
                        let sources =
                            g.g.neighbors_directed(ni, petgraph::Direction::Incoming)
                                .map(|ni| g.g.node_weight(ni).ok_or("huh ? no such node"))
                                .collect::<Result<Vec<_>, _>>()?
                                .into_iter()
                                .map(|x| {
                                    let mut target = g.sandbox.clone();
                                    target.push(x.target().clone());
                                    (target, x.tag().clone())
                                })
                                .collect::<Vec<_>>();

                        let mut target = g.sandbox.clone();
                        target.push(node.target().clone());
                        let sandbox = g.sandbox.clone();

                        let node = node.clone();

                        if g.is_root_node(ni) {
                            let bt = M::BuildType::NotTouched(node.target().clone());
                            match tx.send((ni, bt)).await {
                                Ok(()) => (),
                                Err(e) => {
                                    log::error!("failed to send node index: {ni:?} {e}")
                                }
                            }
                        } else {
                            let tx = tx.clone();
                            let mut logpath = sandbox.clone();

                            let stdout: Result<_, Box<dyn std::error::Error>> = {
                                let mut p = logpath.clone();
                                p.push(format!("{}-stdout.log", node.id()));
                                let parent = p.parent().ok_or::<Box<dyn std::error::Error>>(
                                    format!("cannot get parent for {:?}", p).into(),
                                )?;
                                std::fs::create_dir_all(parent)?;
                                Ok(p)
                            };
                            let stdout: PathBuf = stdout?;

                            let stderr: Result<_, Box<dyn std::error::Error>> = {
                                let mut p = logpath.clone();
                                p.push(format!("{}-stderr.log", node.id()));
                                let parent = p.parent().ok_or::<Box<dyn std::error::Error>>(
                                    format!("cannot get parent for {:?}", p).into(),
                                )?;
                                std::fs::create_dir_all(parent)?;
                                Ok(p)
                            };
                            let stderr: PathBuf = stderr?;
                            set.spawn(async move {
                                logpath.push("log");
                                std::fs::create_dir_all(&logpath).expect("create logs dir");

                                let success = node.build(
                                    sandbox.clone(),
                                    sources.clone(),
                                    vec![],
                                    stdout,
                                    stderr.clone(),
                                );
                                // let bt = match res {
                                //     Ok(success) => {
                                let bt = if success {
                                    // process ran and exited with code 0
                                    M::BuildType::Rebuilt(target)
                                } else {
                                    // process ran and exited with code != 0
                                    M::BuildType::Failed
                                };

                                match tx.send((ni, bt)).await {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!("failed to send node index: {ni:?} {e}")
                                    }
                                }
                            });
                        }

                        // // break 'outer;
                    } else {
                        log::info!(
                            "node not ready : {:?} ; ok_to_start:{}, an_ancestor_failed:{}, an_ancestor_changed:{}",
                            node.target(),
                            ok_to_start,
                            an_ancestor_failed,
                            an_ancestor_changed
                        );
                    }
                }

                // break 'outer;
            }
        }
        log::info!("recv");
        if let Some(li) = rx.recv().await {
            running.remove(&li.0);
            let node = g.g.node_weight(li.0).ok_or("huh, no node?")?;
            match li.1 {
                M::BuildType::Rebuilt(target) => {
                    rebuilt.insert(li.0);
                    built_targets.insert(li.0, target);
                    pb.println(format!(
                        "{} {} {:?} ",
                        id_text(li.0),
                        built_text,
                        node.target()
                    ));
                }
                M::BuildType::NotTouched(target) => {
                    skipped.insert(li.0);
                    built_targets.insert(li.0, target);
                    // pb.println(format!("{} node {:?} ", not_touched_text, node));
                }
                M::BuildType::Failed => {
                    failed.insert(li.0);
                    pb.println(format!(
                        "{} {} node {:?} ",
                        id_text(li.0),
                        failed_text,
                        node.target()
                    ));
                }
                M::BuildType::AncestorFailed => {
                    ancestor_failed.insert(li.0);
                }
            };
            pb.inc(1);
        }
    }

    pb.println(" --- SUMMARY --- ");

    for ni in ancestor_failed {
        let node = g.g.node_weight(ni).ok_or("huh, no node?")?;
        pb.println(format!(
            "{} node {:?} ",
            ancestor_failed_text,
            node.target()
        ));
    }

    for ni in failed {
        let node = g.g.node_weight(ni).ok_or("huh, no node?")?;
        pb.println(format!("{} node {:?} ", failed_text, node.target()));
    }
    pb.finish_with_message("done");

    Ok(())
}

pub async fn scan(g: &mut M::G) -> Result<(), Box<dyn std::error::Error>> {
    let id_text = |id: NodeIndex| -> String {
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

    let mut nodes_to_scan: Vec<(NodeIndex, &Arc<dyn M::GNode>)> = Vec::new();
    for ni in g.g.node_indices() {
        if g.g.edges_directed(ni, Incoming).count() as u32 == 0 {
            let node = &g.g.node_weight(ni).ok_or("huh ?")?;

            nodes_to_scan.push((ni, node));
        }
    }
    let mut edges_to_add: Vec<(NodeIndex, NodeIndex)> = Vec::new();

    for (ni, n) in nodes_to_scan {
        let scanned_deps = &n.scan(g.srcdir.clone(), n.target().clone())?;
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
                    log::warn!("could resolve dep {p:?}");
                    // if a scanned dependency does not exist, then it will not be copied to the sandbox and the build will fail
                }
            }
        }

        let scan_text = "SCANNED".hex("#7FFF00").bold();
        pb.println(format!(
            "{:3} {scan_text} {:?} -> {} new edge(s)",
            id_text(ni),
            &n.target(),
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

    Ok(())
}
