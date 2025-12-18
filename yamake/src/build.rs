// use crate::error as E;
use colored_text::Colorize;
use log;
use petgraph::Direction::Incoming;
use petgraph::dot::Dot;
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::result::Result;
// use std::time::Duration;

use crate::model as M;
use crate::model::PathWithTag;
use crate::target_hash::write_current_hash;
// use tokio::sync::mpsc::Receiver;

pub(crate) async fn build(
    g: &mut M::G,
    _force_rebuild: bool,
    nb_workers: u32,
) -> Result<M::MakeReturn, Box<dyn std::error::Error>> {
    // for ni in g.g.node_indices() {
    //     let n = g.g.node_weight(ni).ok_or("get node")? ;
    //     g.status.insert(n.id(),M::EStatus::Initial) ;
    // }

    let mut ret: HashMap<NodeIndex, M::BuildType> = HashMap::new();

    let pb = ProgressBar::new(g.g.node_indices().count().try_into().unwrap());
    pb.set_style(
        ProgressStyle::with_template(
            ":-) make  [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
        )
        .unwrap(),
    );

    let count = mount(g)?;
    log::info!("{count} nodes are mounted ; {} in total", g.g.node_count());
    g.scan().await?;
    compute_needs_rebuild(g)?;

    let (tx, mut rx) = mpsc::channel::<(NodeIndex, M::BuildType)>(1000);

    let mut set: JoinSet<()> = JoinSet::new();

    // let done_text = "DONE".hex("#8B008B").on_hex("#7FFF00").bold();
    let built_text = "BUILT ".hex("#00FFAA").bold();
    let mounted_text = "MOUNTED changed ".hex("#0044FF").bold();
    let mounted_not_changed_text = "MOUNTED not changed ".hex("#FF00FF").bold();
    let built_but_not_changed_text = "BUILT not changed ".hex("#FF0033").bold();
    let tag_text = |tag: String| tag.as_str().hex("#000033").on_hex("#eeeeee").bold();

    let _not_touched_text = "Skip".hex("#8B008B").on_hex("#7FFFFF").bold();
    let failed_text = "FAILED"
        // .hex("#FF1493")
        // .on_hex("#F0FFFF")
        // .on_hex("#d38a8aff")
        .bold()
        .on_red();
    // let ancestor_failed_text = "Ancestor Failed".hex("#FF8C00").on_hex("#000000").bold();
    let ancestor_failed_text = "Ancestor Failed".hex("#FF8C00").bold();
    let _id_text = |id: NodeIndex| -> String {
        format!("{:3}", id.index())
            .hex("#8B008B")
            .on_hex("#7FFFFF")
            .bold()
    };

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
            log::info!(
                "{total_nodes} == {} + {} + {} + {}",
                rebuilt.len(),
                failed.len(),
                ancestor_failed.len(),
                skipped.len()
            );
            log::info!("condition met to break out of outer loop");
            break 'outermost;
        }
        // 'outer: loop {
        {
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
                    let needs_rebuild =
                        g.needs_rebuild.get(&node.id()).ok_or("rebuilt not found")?;
                    log::info!("inspect needs rebuild : {} ; {needs_rebuild}", node.id());
                    let mut ancestor_targets: Vec<PathBuf> = vec![];

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
                        let pn = g.g.node_weight(p).ok_or("hu, no node")?;
                        ancestor_targets.push(pn.target());
                    }
                    if an_ancestor_failed {
                        log::info!("ANCESTOR FAILED === > {:?} ; {:?}", node.target(), ni);
                        pending.remove(&ni);
                        ancestor_failed.insert(ni);
                        ret.insert(ni, M::BuildType::AncestorFailed.clone());
                        match tx.send((ni, M::BuildType::AncestorFailed)).await {
                            Ok(()) => {
                                // log::info!("ok, sent");
                                ()
                            }
                            Err(e) => {
                                log::error!("failed to send node index: {ni:?} {e}");
                                return Err("failed to send node index: {ni:?} {e}".into());
                            }
                        };
                    } else if ok_to_start && !an_ancestor_changed && !needs_rebuild {
                        log::info!("SKIP === > {:?}", node);
                        pending.remove(&ni);
                        skipped.insert(ni.clone());
                        // hum... why this ?
                        let bt = if g.is_root_node(ni) {
                            M::BuildType::MountNotChanged(node.target().clone())
                        } else {
                            M::BuildType::NotRebuilt(node.target().clone())
                        };
                        ret.insert(ni, bt.clone());
                        match tx.send((ni, bt)).await {
                            Ok(()) => {
                                // log::info!("ok, sent");
                                ()
                            }
                            Err(e) => log::error!("failed to send node index: {:?} {}", ni, e),
                        };
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
                                    M::PathWithTag {
                                        path: target,
                                        tag: x.tag().clone(),
                                    }
                                })
                                .collect::<Vec<_>>();

                        let mut target = g.sandbox.clone();
                        target.push(node.target().clone());
                        let sandbox = g.sandbox.clone();

                        let node = node.clone();

                        if g.is_root_node(ni) {
                            let needs_rebuild =
                                g.needs_rebuild.get(&node.id()).ok_or("huh, no node")?;
                            log::info!(
                                "is root node ; needs rebuild '{:?}' :  {}",
                                needs_rebuild,
                                &node.id()
                            );
                            let bt = if *needs_rebuild {
                                M::BuildType::MountChanged(node.target().clone())
                            } else {
                                M::BuildType::MountNotChanged(node.target().clone())
                            };
                            match tx.send((ni, bt)).await {
                                Ok(()) => (),
                                Err(e) => {
                                    log::error!("failed to send node index: {ni:?} {e}");
                                    return Err("failed to send node index: {ni:?} {e}".into());
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
                            log::info!("stdout is {:?}", stdout);

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

                                let old_digest =
                                    target_hash::get_hash_of_node(sandbox.clone(), node.target())
                                        .unwrap_or(None);
                                log::info!("stdout is {:?}", stdout);
                                let stdout = std::fs::File::create(stdout).expect("create stdout");
                                let stderr = std::fs::File::create(stderr).expect("create stderr");
                                let success = node.build(sandbox.clone(), sources, stdout, stderr);

                                let new_digest =
                                    target_hash::get_hash_of_node(sandbox.clone(), node.target())
                                        .unwrap_or(None);

                                let bt = if success {
                                    // process ran and exited with code 0
                                    log::info!("old digest : {old_digest:?}");
                                    log::info!("new digest : {new_digest:?}");

                                    if old_digest != new_digest {
                                        M::BuildType::Rebuilt(node.target())
                                    } else {
                                        M::BuildType::RebuiltButUnchanged(node.target())
                                    }
                                } else {
                                    // process ran and exited with code != 0
                                    M::BuildType::Failed
                                };

                                match tx.send((ni, bt)).await {
                                    Ok(()) => (),
                                    Err(e) => {
                                        log::error!(
                                            "{}:{} failed to send node index: {ni:?} {e}",
                                            file!(),
                                            line!()
                                        );
                                        // return Err("failed to send node index: {ni:?} {e}".into());
                                        ()
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
            let bt = li.1;
            ret.insert(li.0, bt.clone());
            log::info!("ret is now {:?}", ret);
            match bt {
                M::BuildType::MountChanged(target) => {
                    rebuilt.insert(li.0);
                    built_targets.insert(li.0, target);
                    pb.println(format!(
                        "{} {:?} [{}]",
                        // id_text(li.0),
                        mounted_text,
                        node.target().clone(),
                        tag_text(node.tag()),
                    ));
                }
                M::BuildType::MountNotChanged(target) => {
                    // rebuilt.insert(li.0);
                    built_targets.insert(li.0, target);
                    pb.println(format!(
                        "{} {:?} [{}]",
                        // id_text(li.0),
                        mounted_not_changed_text,
                        node.target().clone(),
                        tag_text(node.tag()),
                    ));
                }
                M::BuildType::Rebuilt(target) => {
                    rebuilt.insert(li.0);
                    built_targets.insert(li.0, target);
                    pb.println(format!(
                        "{} {:?} [{}]",
                        // id_text(li.0),
                        built_text,
                        node.target().clone(),
                        tag_text(node.tag()),
                    ));
                }
                M::BuildType::RebuiltButUnchanged(target) => {
                    skipped.insert(li.0);
                    built_targets.insert(li.0, target);
                    pb.println(format!(
                        "{} {:?} ",
                        // id_text(li.0),
                        built_but_not_changed_text,
                        node.target().clone()
                    ));
                }

                M::BuildType::NotRebuilt(target) => {
                    skipped.insert(li.0);
                    // let node = g.g.node_weight(ni).ok_or("huh?")?;
                    built_targets.insert(li.0, target);
                    // pb.println(format!("{} node {:?} ", not_touched_text, node));
                }
                M::BuildType::Failed => {
                    failed.insert(li.0);
                    pb.println(format!(
                        "{} node {:?} [{}]",
                        // id_text(li.0),
                        failed_text,
                        node.target(),
                        tag_text(node.tag())
                    ));
                }
                M::BuildType::AncestorFailed => {
                    ancestor_failed.insert(li.0);
                }
            };
            pb.inc(1);
        }
    }
    if pending.len()
        + running.len()
        + rebuilt.len()
        + failed.len()
        + ancestor_failed.len()
        + skipped.len()
        != total_nodes
    {
        return Err("counts don't match".into());
    }
    log::info!("got out of outer loop");
    pb.println("writing new hashes");
    // compute_needs_rebuild(&g) ;
    write_current_hash(&g)?;

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

    {
        // structs to write the output build report in json
        #[derive(Debug, Serialize)]
        pub struct ResultItem {
            pub target: String,
            pub stdout: String,
            pub stderr: String,
            pub status: String,
            pub explicit_deps: Vec<String>,
            pub scanned_deps: Vec<String>,
        }

        #[derive(Debug, Serialize)]
        pub struct Result {
            pub srcdir: String,
            pub sandbox: String,
            pub items: Vec<ResultItem>,
        }

        // write the results to a file
        let mut resultpath = g.sandbox.clone();
        resultpath.push("make-report.json");
        // let mut x: Vec<(PathBuf, M::BuildType, usize)> = vec![];
        let mut items: Vec<ResultItem> = vec![];
        for (ni, bt) in ret.iter() {
            let node = g.g.node_weight(*ni).ok_or("huh ?")?;
            log::info!("{} : {:?} => {:?}", node.id(), node.target(), bt);
            let target: String = node
                .target()
                .as_os_str()
                .to_str()
                .expect("target path")
                .into();
            let stdout = node
                .target()
                .as_os_str()
                .to_str()
                .expect("target path")
                .to_string()
                + "-stdout.log";
            let stderr = node
                .target()
                .as_os_str()
                .to_str()
                .expect("target path")
                .to_string()
                + "-stderr.log";
            let mut explicit_deps: Vec<String> = vec![];
            let mut scanned_deps: Vec<String> = vec![];

            for e in g.g.edges_directed(*ni, Incoming) {
                log::info!("{:?}", e);
                log::info!("{:?}", e.weight());
                let n2 = g.g.node_weight(e.source()).ok_or("hugh ?")?;
                log::info!("{:?}", n2.target());
                let p: String = n2
                    .target()
                    .clone()
                    .as_os_str()
                    .to_str()
                    .expect("target path")
                    .into();
                match e.weight().kind {
                    M::EKind::Explicit => explicit_deps.push(p),
                    M::EKind::Scanned => scanned_deps.push(p),
                }
                // unimplemented!();
            }

            let item = ResultItem {
                target: target,
                stdout: stdout,
                stderr: stderr,
                status: bt.to_string(),
                explicit_deps: explicit_deps,
                scanned_deps: scanned_deps,
            };
            items.push(item);
        }
        let result = Result {
            srcdir: g.srcdir.to_str().expect("sandbox").to_string(),
            sandbox: g.sandbox.to_str().expect("sandbox").to_string(),
            items: items,
        };
        std::fs::write(resultpath, serde_json::to_string_pretty(&result)?)?;
    }

    let mut success = true;
    for (_k, v) in ret.iter() {
        match v {
            M::BuildType::Failed => success = false,
            M::BuildType::MountChanged(_)
            | M::BuildType::MountNotChanged(_)
            | M::BuildType::Rebuilt(_)
            | M::BuildType::NotRebuilt(_)
            | M::BuildType::RebuiltButUnchanged(_)
            | M::BuildType::AncestorFailed => {}
        }
    }

    if ret.len() != g.g.node_count() {
        let msg = format!(
            "internal logic error, the map <node,build result> has len {}, but the graph has {} nodes",
            ret.len(),
            g.g.node_count()
        );
        log::error!("{}", msg);
        for ni in g.g.node_indices() {
            if !ret.contains_key(&ni) {
                let node = g.g.node_weight(ni).ok_or("huh ?")?;
                log::error!("missing node {:?} in ret", node.target());
            }
        }

        return Err(msg.into());
    }

    Ok(M::MakeReturn { success, nt: ret })
}
