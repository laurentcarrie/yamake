use colored_text::Colorize;
use log;

use std::path::PathBuf;
use tokio::sync::mpsc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use std::result::Result;
// use std::time::Duration;

use crate::model as M;
// use tokio::sync::mpsc::Receiver;
use tokio::task::JoinSet;

pub fn mount(g: &M::G) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("mount");
    std::fs::create_dir_all(&g.sandbox);

    for id in g.g.node_indices() {
        let n = g.g.node_weight(id).ok_or("huh ?")?;
        if g.g
            .neighbors_directed(id, petgraph::Direction::Incoming)
            .count()
            == 0
        {
            log::info!("mount {:?}", id);
            let n = g.g.node_weight(id).ok_or("huh ?")?;
            let mut target_in_srcdir = g.srcdir.clone();
            target_in_srcdir.push(&n.target);
            if !target_in_srcdir.exists() {
                let msg = format!(
                    r###"""
                this target node has no predecessor : {}
                either :
                - it is a source file that does not exist, check typos or create it
                - it is a built file, add a link between this node and its predecessors
                """###,
                    n.target.display().hex("#FF1493").on_hex("#F0FFFF").bold(),
                );
                return Err(msg.into());
            }
            let mut target_in_sandbox = g.sandbox.clone();
            target_in_sandbox.push(&n.target);

            log::info!("MOUNT {:?} => {:?}", target_in_srcdir, target_in_sandbox);
            std::fs::create_dir_all(target_in_sandbox.parent().ok_or("no parent ?")?)?;
            std::fs::copy(
                target_in_srcdir.clone().as_os_str(),
                target_in_sandbox.as_os_str(),
            )?;
        }
    }

    Ok(true)
}

async fn build_node(
    tx: mpsc::Sender<(NodeIndex, M::BuildType)>,
    sandbox: PathBuf,
    target: PathBuf,
    sources: Vec<(PathBuf, String)>,
    ni: NodeIndex,
    build: M::BuildFn,
) -> () {
    let bt = match (build)(sandbox.clone(), target.clone(), sources) {
        Ok(success) => {
            if success {
                M::BuildType::Rebuilt(target)
            } else {
                M::BuildType::Failed
            }
        }
        Err(e) => {
            log::error!("{}", e);
            M::BuildType::Failed
        }
    };

    match tx.send((ni, bt)).await {
        Ok(()) => (),
        Err(e) => log::error!("failed to send node index: {:?} {}", ni, e),
    };
}

pub async fn make(
    g: &M::G,
    _force_rebuild: bool,
    nb_workers: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    mount(g)?;
    let (tx, mut rx) = mpsc::channel::<(NodeIndex, M::BuildType)>(1000);

    let mut set: JoinSet<()> = JoinSet::new();

    // let g: petgraph::Graph<M::N, M::E> = g.g;

    // let done_text = "DONE".hex("#8B008B").on_hex("#7FFF00").bold();
    let done_text = "DONE".hex("#7FFF00").bold();
    let _not_touched_text = "Skip".hex("#8B008B").on_hex("#7FFFFF").bold();
    let failed_text = "FAILED".hex("#FF1493").on_hex("#F0FFFF").bold();
    let ancestor_failed_text = "Ancestor Failed".hex("#FF8C00").on_hex("#000000").bold();

    let pb = ProgressBar::new(g.g.node_indices().count().try_into().unwrap());
    pb.set_style(
        ProgressStyle::with_template(
            "PDF  [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
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
            log::error!("pending : {:?}", pending);
            log::error!("running : {:?}", running);
            log::error!("rebuilt : {:?}", rebuilt);
            log::error!("failed : {:?}", failed);
            log::error!("ancestor_failed : {:?}", ancestor_failed);
            log::error!("skipped : {:?}", skipped);
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
        'outer: loop {
            // log::info!("running: {:?}", running.len());
            // if running.len() == nb_workers as usize {
            //     log::info!("break 'outer");
            //     break 'outer;
            // }
            if pending.is_empty() {
                break 'outer;
            }

            loop {
                for n in g.g.node_indices() {
                    if running.len() == nb_workers as usize {
                        log::info!("break 'outer");
                        break 'outer;
                    }
                    if !pending.contains(&n) {
                        continue;
                    }
                    let node = g.g.node_weight(n).ok_or("huh, no node?")?;
                    let mut ok_to_start = true;
                    let mut an_ancestor_failed = false;
                    let mut an_ancestor_changed = false;

                    for p in g.g.neighbors_directed(n, petgraph::Direction::Incoming) {
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
                        log::info!("ANCESTOR FAILED === > {:?} ; {:?}", node, n);
                        pending.remove(&n);
                        ancestor_failed.insert(n);
                        match tx.send((n, M::BuildType::AncestorFailed)).await {
                            Ok(()) => {
                                // log::info!("ok, sent");
                                ()
                            }
                            Err(e) => log::error!("failed to send node index: {:?} {}", n, e),
                        };
                    // } else if ok_to_start && !an_ancestor_changed {
                    //     log::info!("SKIP === > {:?}", node);
                    //     pending.remove(&n);
                    //     skipped.insert(n);
                    //     match tx
                    //         .send((n, M::BuildType::NotTouched(PathBuf::from(""))))
                    //         .await
                    //     {
                    //         Ok(()) => {
                    //             // log::info!("ok, sent");
                    //             ()
                    //         }
                    //         Err(e) => log::error!("failed to send node index: {:?} {}", n, e),
                    //     };
                    } else if ok_to_start {
                        log::info!("START === > node {:?} ; id {:?}", node, n);
                        pending.remove(&n);
                        running.insert(n);
                        let sources =
                            g.g.neighbors_directed(n, petgraph::Direction::Incoming)
                                .map(|ni| g.g.node_weight(ni).ok_or("huh ? no such node"))
                                .collect::<Result<Vec<_>, _>>()?
                                .into_iter()
                                .map(|x| (x.target.clone(), x.tag.clone()))
                                .collect::<Vec<_>>();

                        // tx.send(n).await.unwrap();
                        set.spawn(build_node(
                            tx.clone(),
                            g.sandbox.clone(),
                            node.target.clone(),
                            sources,
                            n,
                            node.build,
                        ));
                        // break 'outer;
                    } else {
                        log::info!(
                            "node not ready : {:?} ; ok_to_start:{}, an_ancestor_failed:{}, an_ancestor_changed:{}",
                            node,
                            ok_to_start,
                            an_ancestor_failed,
                            an_ancestor_changed
                        );
                    }
                }
                break 'outer;
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
                    pb.println(format!("{} node {:?} ", done_text, node));
                }
                M::BuildType::NotTouched(target) => {
                    skipped.insert(li.0);
                    built_targets.insert(li.0, target);
                    // pb.println(format!("{} node {:?} ", not_touched_text, node));
                }
                M::BuildType::Failed => {
                    failed.insert(li.0);
                    pb.println(format!("{} node {:?} ", failed_text, node));
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
        pb.println(format!("{} node {:?} ", ancestor_failed_text, node));
    }

    for ni in failed {
        let node = g.g.node_weight(ni).ok_or("huh, no node?")?;
        pb.println(format!("{} node {:?} ", failed_text, node));
    }
    pb.finish_with_message("done");

    Ok(())
}
