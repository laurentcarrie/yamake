use regex::Regex;
use std::path::PathBuf;
// use std::sync::Arc;

pub(crate) fn c_file_scan(
    srcdir: PathBuf,
    target: PathBuf,
    include_paths: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    log::info!("scan {:?}", target);
    let mut src_target = srcdir.clone();
    src_target.push(target);
    if !src_target.exists() {
        return Err(format!("cannot scan non existing file : {:?}", src_target).into());
    }
    let data = std::fs::read_to_string(src_target)?;
    // let re = Regex::new(r###" *\#include *"(?<name>\w+)" *"###)?;
    let re = Regex::new(r###"#include *"(?<f>.*)".*"###)?;

    let mut ret: Vec<PathBuf> = vec![];

    for caps in re.captures_iter(data.as_str()) {
        log::info!("{:?}", caps);
        log::info!("scan ==> {:?}", caps.name("f"));
        let relpath = caps.name("f").ok_or("huh ? in scan")?.as_str();
        log::info!("found rel path {}", relpath);
        'outer: loop {
            for i in &include_paths {
                // let mut scanned = srcdir.clone();
                let mut scanned = i.clone();
                scanned.push(PathBuf::from(relpath));
                log::info!(" try in include path : {:?}", scanned);
                if scanned.exists() {
                    log::info!("found ! : {:?}", scanned);
                    ret.push(relpath.into());
                    break 'outer;
                }
            }

            log::warn!(
                "could not find scanned dep in any include path {:?}",
                relpath
            );
            break 'outer;
        }
    }

    Ok(ret)
}
