use regex::Regex;
use std::path::PathBuf;

pub(crate) fn ly_file_scan(
    sandbox: PathBuf,
    target: PathBuf,
    include_paths: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    log::info!("scan {:?}", target);
    let mut src_target = sandbox.clone();
    src_target.push(target);
    if !src_target.exists() {
        return Err(format!("cannot scan non existing file : {:?}", src_target).into());
    }
    let data = std::fs::read_to_string(src_target)?;
    // let re = Regex::new(r###" *\#include *"(?<name>\w+)" *"###)?;
    let re = Regex::new(r###"input *"(?<f>.*)".*"###)?;

    let mut ret: Vec<PathBuf> = vec![];

    for caps in re.captures_iter(data.as_str()) {
        log::info!("{:?}", caps);
        log::info!("scan ==> {:?}", caps.name("f"));
        let relpathx = caps.name("f").ok_or("huh ? in scan")?.as_str();
        log::info!("found rel path {}", relpathx);
        'outer: loop {
            for i in &include_paths {
                // let mut scanned = srcdir.clone();
                let mut scanned = i.clone();
                scanned.push(PathBuf::from(relpathx));
                log::info!(" try in include path : {:?}", scanned);
                if scanned.exists() {
                    log::info!("found ! : {:?}", scanned);
                    ret.push(relpathx.into());

                    // recursive scan
                    let others = ly_file_scan(sandbox.clone(), scanned, include_paths.clone())?;
                    ret.extend(others);

                    break 'outer;
                }
            }

            log::warn!(
                "could not find scanned dep in any include path {:?}",
                relpathx
            );
            break 'outer;
        }
    }

    Ok(ret)
}
