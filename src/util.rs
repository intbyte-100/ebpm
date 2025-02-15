use std::{
    fs,
    io::{self, Error, ErrorKind, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::input_string;

fn recoursive_copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;

        
        let ty = entry.file_type()?;
        if ty.is_dir() {
            recoursive_copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub(crate) fn copy(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    if let Ok(file) = fs::metadata(&src) {
        if file.is_file() {
            let file = src.as_ref().file_name().unwrap();
            fs::copy(src.as_ref(), dst.as_ref().join(file))?;
            return Ok(());
        }
    }

    let dir = src
        .as_ref()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    
    recoursive_copy(src, dst.as_ref().join(dir))
}

pub(crate) fn remove(path: impl AsRef<Path>) -> io::Result<()> {
    if let Ok(meta) = fs::metadata(&path) {
        match meta.is_dir() {
            true => fs::remove_dir_all(path),
            false => fs::remove_file(path),
        }
    } else {
        Err(Error::from(ErrorKind::NotConnected))
    }
}

pub(crate) fn request_yes_or_no(request: &str) -> bool {
    loop {
        print!("{} {} ", request.green(), "[y/n]:".green());
        std::io::stdout().flush().unwrap();

        let string = input_string();
        let answer = string.trim();

        match answer {
            "y" => return true,
            "n" => return false,
            answer => {
                let message =
                    format!("'{}' is incorrect choice! Requires'y' or 'n'.", answer).red();
                println!("{}", message);
            }
        }
    }
}
