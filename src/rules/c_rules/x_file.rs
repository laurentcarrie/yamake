use crate::model as M;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// implementation of linking object files to get an exe :
/// gcc a.o b.o c.o -o myexe

#[derive(Debug, Clone)]
pub struct N {
    target: PathBuf,
    flags: Vec<String>,
}

pub fn new(target: PathBuf, flags: Vec<String>) -> Result<N, Box<dyn std::error::Error>> {
    // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
    Ok(N { target, flags })
}

impl M::GNode for N {
    fn build(
        &self,
        sandbox: PathBuf,
        sources: Vec<M::PathWithTag>,
        stdout: std::fs::File,
        mut stderr: std::fs::File,
    ) -> bool {
        let mut filtered_sources: Vec<PathBuf> = vec![];
        for x in sources {
            if x.tag != "o file" {
                writeln!(stderr, "found bad source : {:?}", x).expect("write error");
                return false;
            } else {
                filtered_sources.push(x.path);
            }
        }

        let mut binding = Command::new("gcc");
        let binding = binding
            .args(filtered_sources)
            // .arg("-v")
            .arg("-o")
            .arg(self.target())
            .args(self.flags.clone())
            .current_dir(&sandbox)
            .stdout(stdout)
            .stdout(stderr);
        let child = binding;
        log::info!("child is : {:?}", &child);
        log::info!("exit : {:?}", child.status());

        match child.status() {
            Ok(e) => e.success(),
            Err(_e) => {
                // writeln!(stderr, "{:?}", e).expect("write error");
                false
            }
        }
    }

    fn target(&self) -> PathBuf {
        PathBuf::from(self.target.clone())
    }
    fn tag(&self) -> String {
        "x file".to_string()
    }
    fn id(&self) -> String {
        self.target().to_str().expect("target to str").to_string()
    }
}
