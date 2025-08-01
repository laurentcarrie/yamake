use std::path::PathBuf;
use std::process::Command;

pub(crate) fn object_file_from_cfile(
    sandbox: PathBuf,
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
    include_paths: Vec<PathBuf>,
    stdout: PathBuf,
    stderr: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    log::info!("compile C file {:?}", target_file);
    if sources.len() != 1 {
        return Err("bad length of sources, should be 1".into());
    };
    let source = sources.first().ok_or("empty sources")?;
    if source.1 != "c file".to_string() {
        return Err("source has bad tag".into());
    };
    let source = source.0.clone();

    log::info!("compile, source is : {:?}", source.file_name());
    log::info!("compile, target : {:?}", target_file);

    let mut binding = Command::new("gcc");
    let mut binding = binding
        .arg("-c")
        .arg(source)
        .arg("-o")
        .arg(target_file)
        .current_dir(&sandbox)
        .stdout(std::fs::File::create(stdout)?)
        .stderr(std::fs::File::create(stderr)?);
    for pi in include_paths {
        binding = binding.arg("-I").arg(pi);
    }
    let child = binding;
    log::info!("{:?}", child);
    if child.status()?.success() {
        Ok(true)
    } else {
        log::error!("child is : {:?}", &child);
        log::error!("exit : {:?}", child.status());
        Ok(false)
    }
}
