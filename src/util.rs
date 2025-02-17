use std::io::Write;

use colored::Colorize;

use crate::input_string;

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
