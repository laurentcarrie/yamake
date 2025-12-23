use std::path::PathBuf;

pub fn string_of_path(p: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let s = p
        .as_path()
        .to_str()
        .ok_or("huh, cannot get string of path")?;
    Ok(String::from(s))
}

/// if root is `/a/b/c/d`
/// and p is `/a/b/c/d/e/f`
/// will return the relative path : `e/f`
pub fn relpath(root: PathBuf, p: PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if root.is_relative() {
        return Err("rootdir is relative".into());
    }

    if p.is_relative() {
        return Err("abspath is relative".into());
    }

    // let binding = rootdir.canonicalize().unwrap();
    // let s1 = binding.to_str().unwrap();
    let s1 = root.to_str().unwrap();
    let s2 = p
        // .canonicalize()
        // .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    if !s2.starts_with(s1) {
        return Err(format!("cannot make relpath {:?} is not in {:?}", root, p).into());
    }
    let s3 = s2.replace(format!("{s1}/").as_str(), "");
    log::debug!("s3 : {:?}", s3);
    Ok(PathBuf::from(s3))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relpath() {
        let root = PathBuf::from("/a/b/c/d/e");
        let f = PathBuf::from("/a/b/c/d/e/f/h.tex");
        let r = relpath(root, f).unwrap();
        let expected = PathBuf::from("f/h.tex");

        assert_eq!(r, expected);
    }

    #[test]
    #[should_panic(
        expected = "called `Result::unwrap()` on an `Err` value: \"rootdir is relative\""
    )]
    fn test_root_not_absolute() -> () {
        let root = PathBuf::from("a");
        let f = PathBuf::from("/f/g");
        let _r = relpath(root, f).unwrap();

        ()
    }

    #[test]
    #[should_panic(expected = "\"cannot make relpath \\\"/a\\\" is not in \\\"/f/g\\\"")]

    fn test_failed_relpath() -> () {
        let root = PathBuf::from("/a");
        let f = PathBuf::from("/f/g");
        let r = relpath(root, f).unwrap();
        log::info!("r : {:?}", r);

        ()
    }
}
