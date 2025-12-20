use log;
use std::collections::HashMap;

use crate::model as M;

pub(crate) async fn make(
    g: &mut M::G,
    force_rebuild: bool,
    nb_workers: u32,
) -> Result<M::MakeReturn, Box<dyn std::error::Error>> {
    log::info!("make called with nb_workers={}", nb_workers);
    g.mount()?;
    'outer: loop {
        log::info!("make iteration");
        'inner: loop {
            let changed = g.expand().await?;
            log::info!("expand found new nodes : {changed}");
            if !changed {
                break 'inner;
            }
            log::info!("expand found new nodes, rescanning");
        }
        let changed = g.scan().await?;
        if !changed {
            log::info!("scan is done");
            break 'outer;
        }
        let ret = g.build(force_rebuild, nb_workers).await?;
        break;
    }
    let ret = M::MakeReturn {
        success: false,
        nt: HashMap::new(),
    };
    log::info!("make finished");
    Ok(ret)
}
