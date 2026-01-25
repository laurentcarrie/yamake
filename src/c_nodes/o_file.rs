use crate::command::run_command;
use crate::model::GNode;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct OFile {
    pub name: String,
    pub include_paths: Vec<PathBuf>,
    pub compile_flags: Vec<String>,
}

fn scan_file_recursive(
    srcdir: &Path,
    file_path: &Path,
    include_re: &Regex,
    result: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) {
    let content = fs::read_to_string(file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e));

    for cap in include_re.captures_iter(&content) {
        let header = &cap[1];
        let header_path = PathBuf::from(header);

        // Skip if already visited
        if visited.contains(&header_path) {
            continue;
        }

        let src_header = srcdir.join(&header_path);

        if !src_header.exists() {
            panic!(
                "Header file not found: {} (looked in {})",
                header,
                src_header.display()
            );
        }

        visited.insert(header_path.clone());
        result.push(header_path);

        // Recursively scan the header file
        scan_file_recursive(srcdir, &src_header, include_re, result, visited);
    }
}

impl OFile {
    pub fn new(name: &str, include_paths: Vec<PathBuf>, compile_flags: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            include_paths,
            compile_flags,
        }
    }
}

impl GNode for OFile {
    fn build(&self, sandbox: &Path, predecessors: &[&(dyn GNode + Send + Sync)]) -> bool {
        let inputs: Vec<PathBuf> = predecessors
            .iter()
            .filter(|p| p.tag() != "HFile")
            .map(|p| sandbox.join(p.pathbuf()))
            .collect();

        let mut cmd = Command::new("gcc");
        cmd.arg("-c");
        for flag in &self.compile_flags {
            cmd.arg(flag);
        }
        cmd.arg("-I").arg(sandbox);
        for include_path in &self.include_paths {
            cmd.arg("-I").arg(sandbox.join(include_path));
        }
        cmd.arg("-o").arg(sandbox.join(&self.name));
        for input in &inputs {
            cmd.arg(input);
        }

        run_command(&mut cmd, sandbox, &self.name)
    }

    fn scan(
        &self,
        srcdir: &Path,
        predecessors: &[&(dyn GNode + Send + Sync)],
    ) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let include_re = Regex::new(r#"(?m)^\s*#include\s+"([^"]+)""#).unwrap();

        // Scan C files for includes
        for pred in predecessors.iter().filter(|p| p.tag() == "CFile") {
            let file_path = srcdir.join(pred.pathbuf());
            scan_file_recursive(srcdir, &file_path, &include_re, &mut result, &mut visited);
        }

        result
    }

    fn id(&self) -> String {
        self.name.clone()
    }

    fn tag(&self) -> String {
        "OFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
