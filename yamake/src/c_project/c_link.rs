use std::path::PathBuf;
use std::process::Command;

pub fn exe_from_obj_files(
    sandbox: PathBuf,
    target_file: PathBuf,
    sources: Vec<(PathBuf, String)>,
    stdout: PathBuf,
    stderr: PathBuf,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut binding = Command::new("gcc");
    let binding = binding
        .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
        .arg("-o")
        .arg(target_file)
        .current_dir(&sandbox)
        .current_dir(&sandbox)
        .stdout(std::fs::File::create(stdout)?)
        .stderr(std::fs::File::create(stderr)?);
    let child = binding;
    log::info!("child is : {:?}", &child);
    log::info!("exit : {:?}", child.status());

    Ok(true)
}
