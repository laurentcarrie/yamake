use log::info;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Creates the log directory structure and returns the stdout/stderr file paths.
fn create_log_paths(
    sandbox: &Path,
    node_id: &str,
) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    let logs_dir = sandbox
        .join("logs")
        .join(Path::new(node_id).parent().unwrap_or(Path::new("")));
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        log::error!(
            "Failed to create logs directory {}: {}",
            logs_dir.display(),
            e
        );
        return None;
    }

    let log_base = sandbox.join("logs").join(node_id);
    let stdout_path = log_base.with_file_name(format!(
        "{}.stdout",
        log_base.file_name().unwrap_or_default().to_string_lossy()
    ));
    let stderr_path = log_base.with_file_name(format!(
        "{}.stderr",
        log_base.file_name().unwrap_or_default().to_string_lossy()
    ));

    Some((stdout_path, stderr_path))
}

/// Logs the output of a pure-Rust build operation.
///
/// Use this for build operations that don't run external commands but still
/// want to create log files for consistency.
///
/// Log files are written to `<sandbox>/logs/<node_id>.stdout` and `<sandbox>/logs/<node_id>.stderr`.
///
/// # Arguments
/// * `sandbox` - The sandbox directory
/// * `node_id` - The node identifier (used for log file naming)
/// * `description` - A description of the build operation (written to stdout log)
/// * `stdout` - Content to write to stdout log
/// * `stderr` - Content to write to stderr log (use empty string if no errors)
///
/// # Returns
/// Returns true if log files were created successfully.
pub fn log_build(
    sandbox: &Path,
    node_id: &str,
    description: &str,
    stdout: &str,
    stderr: &str,
) -> bool {
    let Some((stdout_path, stderr_path)) = create_log_paths(sandbox, node_id) else {
        return false;
    };

    // Write stdout
    if let Ok(mut file) = File::create(&stdout_path) {
        let _ = writeln!(file, "{description}");
        let _ = file.write_all(stdout.as_bytes());
    }

    // Write stderr
    if let Ok(mut file) = File::create(&stderr_path) {
        let _ = writeln!(file, "{description}");
        let _ = file.write_all(stderr.as_bytes());
    }

    true
}

/// Runs a command and captures stdout/stderr to log files.
///
/// Log files are written to `<sandbox>/logs/<node_id>.stdout` and `<sandbox>/logs/<node_id>.stderr`.
/// Returns true if the command succeeded, false otherwise.
pub fn run_command(cmd: &mut Command, sandbox: &Path, node_id: &str) -> bool {
    info!("Running: {cmd:?}");

    let Some((stdout_path, stderr_path)) = create_log_paths(sandbox, node_id) else {
        return false;
    };

    match cmd.output() {
        Ok(output) => {
            // Write stdout (with command as first line)
            if let Ok(mut file) = File::create(&stdout_path) {
                let _ = writeln!(file, "{cmd:?}");
                let _ = file.write_all(&output.stdout);
            }

            // Write stderr (with command as first line)
            if let Ok(mut file) = File::create(&stderr_path) {
                let _ = writeln!(file, "{cmd:?}");
                let _ = file.write_all(&output.stderr);
            }

            output.status.success()
        }
        Err(e) => {
            log::error!("Failed to execute command: {e}");
            false
        }
    }
}
