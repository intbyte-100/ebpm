use std::{fs, io::Write, path::PathBuf};


pub(crate) fn request_yes_or_no(request: &str) -> bool {
    loop {
        print!("{} {} ", request, "[y/n]:");
        std::io::stdout().flush().unwrap();

        let string = input_string();
        let answer = string.trim();

        match answer {
            "y" => return true,
            "n" => return false,
            answer => {
                let message =
                    format!("'{}' is incorrect choice! Requires'y' or 'n'.", answer);
                println!("{}", message);
            }
        }
    }
}

#[inline]
pub(crate) fn input_string() -> String {
    let mut string = String::new();

    std::io::stdin().read_line(&mut string).unwrap();
    string
}


pub trait GetSize {
    fn get_size(&self) -> u64;
}

impl GetSize for PathBuf {
    fn get_size(&self) -> u64 {
        let mut size = 0;
    
        if let Ok(entries) = fs::read_dir(self.as_path()) {
            for entry in entries.filter_map(|e| e.ok()) {
                let metadata = match fs::metadata(entry.path()) {
                    Ok(m) => m,
                    Err(_) => continue, 
                };
    
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += entry.path().get_size(); 
                }
            }
        }
    
        size
    }
}