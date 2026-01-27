use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn mount(srcdir: &Path, sandbox: &Path, p: &Path) -> io::Result<()> {
    let src_path = srcdir.join(p);
    let dest_path = sandbox.join(p);

    // Create parent directories if they don't exist
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(&src_path, &dest_path)?;
    Ok(())
}
