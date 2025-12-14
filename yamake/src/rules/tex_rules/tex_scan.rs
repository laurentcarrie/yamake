use crate::helpers::path::relpath;
use regex::Regex;
use std::path::PathBuf;

// ANCHOR: tex_file_scan
pub(crate) fn tex_file_scan(
    srcdir: PathBuf,
    target: PathBuf,
    include_paths: Vec<PathBuf>,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Box<dyn std::error::Error>> {
    log::info!("scan {:?} ; {:?}", target, include_paths);
    let mut src_target = srcdir.clone();
    src_target.push(target);
    if !src_target.exists() {
        return Ok((vec![], vec![src_target]));
        // return Err(format!("cannot scan non existing file : {:?}", src_target).into());
    }
    let data = std::fs::read_to_string(src_target)?;
    // let re = Regex::new(r###" *\#include *"(?<name>\w+)" *"###)?;
    let re = Regex::new(r###"input *\{ *(?<f>.*) *\}.*"###)?;

    let mut found_deps: Vec<PathBuf> = vec![];
    let mut not_found_deps: Vec<PathBuf> = vec![];

    for caps in re.captures_iter(data.as_str()) {
        log::info!("{:?}", caps);
        log::info!("scan ==> {:?}", caps.name("f"));
        let scanned_dep = caps.name("f").ok_or("huh ? in scan")?.as_str();
        log::info!("found rel path {}", scanned_dep);
        'outer: loop {
            for i in &include_paths {
                // let mut scanned = srcdir.clone();
                let mut try_full_path_scanned_dep = i.clone();
                try_full_path_scanned_dep.push(PathBuf::from(scanned_dep));
                log::info!(" try in include path : {:?}", try_full_path_scanned_dep);
                if try_full_path_scanned_dep.exists() {
                    let relative_to_sandbox =
                        relpath(srcdir.clone(), try_full_path_scanned_dep.clone())?;
                    log::info!(
                        "found ! : {:?} ; {:?} ; {:?}",
                        try_full_path_scanned_dep,
                        scanned_dep,
                        relative_to_sandbox
                    );

                    let dep: PathBuf = relative_to_sandbox.clone().into();
                    if dep.exists() {
                        found_deps.push(relative_to_sandbox.clone().into());
                    } else {
                        not_found_deps.push(relative_to_sandbox.clone().into());
                    }

                    // recursive scan
                    let others =
                        tex_file_scan(srcdir.clone(), relative_to_sandbox, include_paths.clone())?;
                    found_deps.extend(others.0);
                    not_found_deps.extend(others.1);

                    break 'outer;
                }
            }

            log::warn!(
                "could not find scanned dep in any include path {:?}",
                scanned_dep
            );
            break 'outer;
        }
    }

    Ok((found_deps, not_found_deps))
}
// ANCHOR_END: tex_file_scan
