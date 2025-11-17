use std::path::Path;
use std::path::PathBuf;

pub fn relpath(rootdir: PathBuf, abspath: PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // if rootdir is /a/b/c/d
    // and abspath is /ab/c/d/e/f
    // will return e/f

    // if rootdir.is_relative() {
    //     return Err("rootdir is relative".into());
    // }

    // if abspath.is_relative() {
    //     return Err("abspath is relative".into());
    // }

    // let binding = rootdir.canonicalize().unwrap();
    // let s1 = binding.to_str().unwrap();
    let s1 = rootdir.to_str().unwrap();
    let s2 = abspath
        // .canonicalize()
        // .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let s3 = s2.replace(format!("{s1}/").as_str(), "");
    log::info!("s3 : {:?}", s3);
    Ok(PathBuf::from(s3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relpath() {
        simple_logging::log_to_stderr(log::LevelFilter::Info);

        let root = PathBuf::from("/a/b/c/d/e");
        let f = PathBuf::from("/a/b/c/d/e/f/h.tex");
        let r = relpath(root, f).unwrap();
        let expected = PathBuf::from("f/h.tex");

        assert_eq!(r,expected) ;
    }
}
