use crate::model as M;
use std::path::PathBuf;
use std::process::Command;

/// implementation of linking object files to get an exe :
/// gcc a.o b.o c.o -o myexe

#[derive(Debug, Clone)]
pub struct Xfile {
    target: PathBuf,
}

impl Xfile {
    pub fn new(target: PathBuf) -> Result<Xfile, Box<dyn std::error::Error>> {
        // let target = target.as_os_str().to_str().ok_or("bad string")?.to_string();
        Ok(Xfile { target })
    }
}

impl M::GNode for Xfile {
    fn build(
        &self,
        sandbox: PathBuf,
        sources: Vec<M::PathWithTag>,
        stdout: PathBuf,
        stderr: PathBuf,
    ) -> bool {
        let mut filtered_sources: Vec<PathBuf> = vec![];
        for x in sources {
            if x.tag != "o file" {
                std::fs::write(stderr, format!("found bad source : {:?}", x)).expect("write error");
                return false;
            } else {
                filtered_sources.push(x.path);
            }
        }

        let mut binding = Command::new("gcc");
        let binding = binding
            .args(filtered_sources)
            .arg("-o")
            .arg(self.target())
            .current_dir(&sandbox)
            .stdout(std::fs::File::create(stdout).expect("stdout"))
            .stderr(std::fs::File::create(&stderr).expect("stderr"));
        let child = binding;
        log::info!("child is : {:?}", &child);
        log::info!("exit : {:?}", child.status());

        match child.status() {
            Ok(e) => e.success(),
            Err(e) => {
                std::fs::write(stderr, format!("{:?}", e)).expect("write error");
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
