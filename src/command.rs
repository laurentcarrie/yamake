use log::info;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// Runs a command and captures stdout/stderr to log files.
///
/// Log files are written to `<sandbox>/logs/<node_id>.stdout` and `<sandbox>/logs/<node_id>.stderr`.
/// Returns true if the command succeeded, false otherwise.
pub fn run_command(cmd: &mut Command, sandbox: &Path, node_id: &str) -> bool {
    info!("Running: {cmd:?}");

    // Create logs directory structure
    let logs_dir = sandbox
        .join("logs")
        .join(Path::new(node_id).parent().unwrap_or(Path::new("")));
    if let Err(e) = fs::create_dir_all(&logs_dir) {
        log::error!(
            "Failed to create logs directory {}: {}",
            logs_dir.display(),
            e
        );
        return false;
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
