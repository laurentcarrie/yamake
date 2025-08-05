// #[serde (serialize,deserialize)]
use crate::model as M;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub(crate) struct H {
    stored: Option<String>,
    on_disk: Option<String>,
    needs_rebuild: bool,
}

pub(crate) fn get_hash_of_node(
    sandbox: PathBuf,
    target: PathBuf,
    preds: Vec<PathBuf>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut ftarget = sandbox.clone();
    ftarget.push(target);
    let hash = if ftarget.exists() {
        let contents = std::fs::read(ftarget)?;
        let mut hasher = Sha256::new();

        hasher.update(contents);
        let result = hasher.finalize();
        let hash = hex::encode(&result[..]);
        Some(hash)
    } else {
        None
    };
    Ok(hash)
}

pub(crate) fn get_current_hash(
    g: &M::G,
) -> Result<HashMap<String, Option<String>>, Box<dyn std::error::Error>> {
    let mut all: HashMap<String, Option<String>> = HashMap::new();
    for ni in g.g.node_indices() {
        let node = g.g.node_weight(ni).ok_or("what")?;
        let hash = get_hash_of_node(g.sandbox.clone(), node.target(), vec![])?;
        all.insert(node.id(), hash);
    }

    Ok(all)
}

pub(crate) fn write_current_hash(g: &M::G) -> Result<(), Box<dyn std::error::Error>> {
    let all = get_current_hash(&g)?;
    let data = serde_json::to_string(&all)?;
    let mut p = g.sandbox.clone();
    p.push("hash.json");
    std::fs::write(p, data)?;
    Ok(())
}

pub(crate) fn get_stored_hash(
    g: &M::G,
) -> Result<HashMap<String, Option<String>>, Box<dyn std::error::Error>> {
    log::info!("get stored hash");
    let mut p = g.sandbox.clone();
    p.push("hash.json");
    let data = if p.exists() {
        log::info!("{}:{}", file!(), line!());
        let data = std::fs::read_to_string(p)?;
        log::info!("{}:{} {}", file!(), line!(), data.len());

        let json = serde_json::from_str::<HashMap<String, Option<String>>>(&data)?;
        log::info!("{}:{}", file!(), line!());

        json
    } else {
        HashMap::<String, Option<String>>::new()
    };
    Ok(data)
}

pub(crate) fn compute_needs_rebuild(g: &mut M::G) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("compute rebuild");
    let s = get_stored_hash(&g)?;
    let c = get_current_hash(&g)?;
    let mut r: HashMap<String, H> = HashMap::new();
    for (k, v) in c {
        let stored = s.get(&k);
        let stored = if let Some(x) = stored {
            x.clone()
        } else {
            None
        };
        let needs_rebuild = match (&v, stored.clone()) {
            (None, _) => true,
            (_, None) => true,
            (Some(a), Some(b)) => *a != b,
        };
        let x = H {
            on_disk: v.clone(),
            stored: stored.clone(),
            needs_rebuild,
        };
        r.insert(k, x);
    }
    let data = serde_json::to_string(&r)?;
    let mut p = g.sandbox.clone();
    p.push("diff.json");
    std::fs::write(p, data)?;
    g.needs_rebuild = HashMap::new();
    for (k, v) in &r {
        log::info!("needs rebuild {} : {}", k, v.needs_rebuild);
        g.needs_rebuild.insert(k.clone(), v.needs_rebuild);
    }
    Ok(())
}
