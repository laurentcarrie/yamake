use std::path::PathBuf;
// use tokio::process::Command;
use crate::c_project::c_scan::c_file_scan;
use crate::model as M;

#[derive(Debug, Clone)]
pub struct Cfile {
    target: PathBuf,
    include_paths: Vec<PathBuf>,
}

impl Cfile {
    pub fn new(
        target: PathBuf,
        include_paths: Vec<PathBuf>,
    ) -> Result<Cfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Cfile {
            target,
            include_paths,
        })
    }
}

impl M::GNode for Cfile {
    fn build(
        &self,
        _sandbox: PathBuf,
        _sources: Vec<PathBuf>,
        _deps: Vec<PathBuf>,
        _stdout: PathBuf,
        _stderr: PathBuf,
    ) -> bool {
        unimplemented!()
    }

    fn scan(
        &self,
        srcdir: PathBuf,
        source: PathBuf,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let ret = c_file_scan(srcdir, source, self.include_paths.clone())?;
        Ok(ret)
    }

    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "c file".to_string()
    }
}

// impl std::fmt::Debug for N {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Node")
//             .field("target", &self.target)
//             .field("tag", &self.tag)
//             .finish()
//     }
// }

// pub fn object_file_from_cfile(
//     sandbox: PathBuf,
//     target_file: PathBuf,
//     sources: Vec<(PathBuf, String)>,
//     stdout: PathBuf,
//     stderr: PathBuf,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     log::info!("compile C file {:?}", target_file);
//     if sources.len() != 1 {
//         return Err("bad length of sources, should be 1".into());
//     };
//     let source = sources.first().ok_or("empty sources")?;
//     if source.1 != "c file".to_string() {
//         return Err("source has bad tag".into());
//     };
//     let source = source.0.clone();

//     log::info!("compile, source is : {:?}", source.file_name());
//     log::info!("compile, target : {:?}", target_file);

//     let mut binding = Command::new("gcc");
//     let binding = binding
//         .arg("-c")
//         .arg(source)
//         .arg("-o")
//         .arg(target_file)
//         .current_dir(&sandbox)
//         .stdout(std::fs::File::create(stdout)?)
//         .stderr(std::fs::File::create(stderr)?);
//     let child = binding;
//     if child.status()?.success() {
//         Ok(true)
//     } else {
//         log::error!("child is : {:?}", &child);
//         log::error!("exit : {:?}", child.status());
//         Ok(false)
//     }
// }

// pub fn exe_from_obj_files(
//     sandbox: PathBuf,
//     _id: NodeIndex,
//     target_file: PathBuf,
//     sources: Vec<(PathBuf, String)>,
//     stdout: PathBuf,
//     stderr: PathBuf,
// ) -> Result<bool, Box<dyn std::error::Error>> {
//     let mut binding = Command::new("gcc");
//     let binding = binding
//         .args(sources.iter().map(|(s, _)| s).collect::<Vec<_>>())
//         .arg("-o")
//         .arg(target_file)
//         .current_dir(&sandbox)
//         .current_dir(&sandbox)
//         .stdout(std::fs::File::create(stdout)?)
//         .stderr(std::fs::File::create(stderr)?);
//     let child = binding;
//     log::info!("child is : {:?}", &child);
//     log::info!("exit : {:?}", child.status());

//     Ok(true)
// }
