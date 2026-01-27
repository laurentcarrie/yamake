use crate::command::run_command;
use crate::model::GNode;
use log::{info, warn};
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
    sandbox: &Path,
    file_path: &Path,
    include_re: &Regex,
    include_paths: &[PathBuf],
    result: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    scan_complete: &mut bool,
) {
    // If file doesn't exist, mark scan as incomplete
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => {
            warn!("while scanning, file not found: {}", file_path.display());
            *scan_complete = false;
            return;
        }
    };

    for cap in include_re.captures_iter(&content) {
        let header = &cap[1];
        let header_path = PathBuf::from(header);

        // Skip if already visited
        if visited.contains(&header_path) {
            continue;
        }

        visited.insert(header_path.clone());
        result.push(header_path.clone());

        // Try to find the header file in sandbox or include paths
        let sandbox_header = sandbox.join(&header_path);
        let found_path = if sandbox_header.exists() {
            Some(sandbox_header)
        } else {
            // Check include paths
            include_paths.iter()
                .map(|p| p.join(&header_path))
                .find(|p| p.exists())
        };

        // Recursively scan the header file if found
        if let Some(actual_path) = found_path {
            scan_file_recursive(sandbox, &actual_path, include_re, include_paths, result, visited, scan_complete);
        } else {
            // File doesn't exist in sandbox or include paths - might be generated later
            warn!("cannot scan missing file: {} (not in sandbox or include paths)", header_path.display());
            *scan_complete = false;
        }
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
        // Only include CFile predecessors as source files - headers are included via -I
        let inputs: Vec<PathBuf> = predecessors
            .iter()
            .filter(|p| p.tag() == "CFile")
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
        sandbox: &Path,
        predecessors: &[&(dyn GNode + Send + Sync)],
    ) -> (bool, Vec<PathBuf>) {
        info!("scan {}",self.pathbuf().display()) ;
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut scan_complete = true;
        let include_re = Regex::new(r#"(?m)^\s*#include\s+"([^"]+)""#).unwrap();

        // Scan C files for includes
        for pred in predecessors.iter().filter(|p| p.tag() == "CFile") {
            let file_path = sandbox.join(pred.pathbuf());
            scan_file_recursive(sandbox, &file_path, &include_re, &self.include_paths, &mut result, &mut visited, &mut scan_complete);
        }

        if !scan_complete {
            info!("scan incomplete for {}", self.pathbuf().display());
        }

        (scan_complete, result)
    }

    fn tag(&self) -> String {
        "OFile".to_string()
    }

    fn pathbuf(&self) -> PathBuf {
        PathBuf::from(&self.name)
    }
}
