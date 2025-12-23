use std::fs;
use std::path::PathBuf;

pub fn create_dir_all(p: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match fs::create_dir_all(p) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{:?}, {:?}", &e, &p).into()),
    }
}

pub fn write(p: &PathBuf, bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    match fs::write(p, bytes) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{:?}, {:?}", &e, &p).into()),
    }
}

pub fn write_string(p: &PathBuf, data: &String) -> Result<(), Box<dyn std::error::Error>> {
    log::debug!("{}:{} write string to {:?}", file!(), line!(), p);
    match fs::write(p, data) {
        Ok(()) => Ok(()),
        Err(e) => Err(format!("{:?}, {:?}", &e, &p).into()),
    }
}

pub fn read_to_string(p: &PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    let str_path = p
        .as_path()
        .to_str()
        .ok_or("huh, path is none ?".to_string())?;
    match fs::read_to_string(str_path) {
        Ok(s) => Ok(s),
        Err(e) => {
            log::debug!("{}:{} {:?}", file!(), line!(), &p);
            Err(format!("{:?},reading {:?}", &e, &p).into())
        }
    }
}

pub fn read_to_vec_u8(p: &PathBuf) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match fs::read(p.as_path()) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("{:?},reading {:?}", &e, &p).into()),
    }
}
pub fn copy_file(pfrom: &PathBuf, pto: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    match fs::copy(pfrom.to_str().unwrap(), pto.to_str().unwrap()) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{:?},copy from {:?} to {:?}", &e, &pfrom, &pto).into()),
    }
}
