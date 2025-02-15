mod program;
mod util;
mod package;
mod filetransfer;

use std::{
    collections::HashSet, env, fs, io::{BufReader, Read, Write}, path::Path, process::{exit, Command}
};

use fs_extra::dir::get_size;
use program::Program;
use util::request_yes_or_no;

use crate::{program::ProgramResources, util::remove};

fn print_help() {
    println!("Usage: ebpm [command] [program_name]");
    println!();
    println!("Commands:");
    println!("    new [program_name] - create a new program file");
    println!("    install [program_name] - install a program");
    println!("    remove [program_name] - remove a program");
    println!("    list - list all installed programs");
    println!("    run [program_name] - run a program");
    println!();
    println!("Example: ebpm new my_program");
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_help();
        return;
    }


    
    match args[1].as_str() {
        "new" => new_program(&args),
        "help" => print_help(),
        "install" => install_program(&args),
        "remove" => remove_program(&args),
        "run" => run_program(&args),
        "list" => print_list(&args),

        _ => {
            println!("Invalid arguments");
            print_help();
            exit(-1);
        }
    }
}

#[inline]
pub(crate) fn input_string() -> String {
    let mut string = String::new();
    std::io::stdin().read_line(&mut string).unwrap();
    string
}

fn new_program(args: &Vec<String>) {
    if args.len() != 3 {
        println!("Error: incrorrect program name specified");
        exit(-1)
    }

    let program = Program::new(args[2].clone(), &Vec::new());
    let json = serde_json::to_string_pretty(&program).unwrap();

    {
        std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(args[2].to_string() + ".json")
            .unwrap_or_else(|_e| fs::File::create(args[2].to_string() + ".json").unwrap())
            .write(json.as_str().as_bytes())
            .unwrap();
    }

    Command::new("nano")
        .arg(args[2].to_string() + ".json")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    let install = request_yes_or_no("Do you want to install program?");

    if install {
        let path = env::current_dir().unwrap();
        let mut program = install_program_from_unpacked_files(path.as_path(), args[2].as_str());
        if request_yes_or_no("Do you want to remove package files?") {
            println!("Removing files:");
            program.files.push(args[2].as_str().to_string() + ".json");
            program.files.iter().for_each(|file| {
                print!("    removing {}... ", file);
                std::io::stdout().flush().unwrap();
                match remove(file) {
                    Ok(_) => println!("[Success]"),
                    Err(e) => println!("[Error: {}]", e.kind()),
                }
            });
        }
    }
}

pub(crate) fn install_program(args: &Vec<String>) {
    let dir = env::current_dir().unwrap();

    if args.iter().filter(|it| it.as_str() == "-f").count() > 0 {
        args.iter()
            .skip(2)
            .filter(|t| t.as_str() != "-fa")
            .for_each(|it| {
                install_program_from_unpacked_files(Path::new(dir.as_path()), it.as_str());
            })
    }
}

fn install_program_from_unpacked_files(path: &Path, name: &str) -> Program {
    let dir = path.to_path_buf();
    let program = get_program_from_path(&dir, name);

    program.install();
    program
}

fn remove_program(args: &Vec<String>) {
    Program::load(&args[2]).remove();
}

fn print_list(args: &Vec<String>) {
    let mut path = std::env::home_dir().unwrap();
    path.push("Applications");
    let files = fs::read_dir(path).unwrap();

    print!("Installed programs:");
    for i in 0..44 - ("Installed programs:".len()) {
        print!(" ");
    }
    println!("Size:");
    files
        .map(|it| it.unwrap().file_name().into_string().unwrap())
        .filter(|it| it.ends_with(".json"))
        .for_each(|it| {
            let program = Program::load(it.strip_suffix(".json").unwrap());
            let resourse = ProgramResources::from(&program);

            print!("--- {}", program.name);

            for _ in 0..40 - program.name.len() {
                print!(" ")
            }
            let folder_size = get_size(resourse.res_path).unwrap();
            let folder_size = if folder_size < 1024 {
                format!("Bytes {}", folder_size)
            } else if folder_size < 1024 * 1024 {
                format!("KiB {:.2}", (folder_size as f64 / 1024.0))
            } else if folder_size < 1024 * 1024 * 1024 {
                format!("MiB {:.2}", (folder_size as f64 / 1024.0 / 1024.0))
            } else if folder_size < 1024 * 1024 * 1024 * 1024 {
                format!("GiB {:.2}", (folder_size as f64 / 1024.0 / 1024.0 / 1024.0))
            } else {
                format!("Bytes {}", folder_size)
            };
            println!("{}", folder_size)
        })
}

fn get_program_from_path(path: &Path, name: &str) -> Program {
    let mut path = path.to_path_buf();
    path.push(name.to_string() + ".json");

    let file = std::fs::File::open(path).unwrap_or_else(|_e| {
        println!("Program '{}' doesn't exist!", name);
        exit(-1);
    });
    let mut reader = BufReader::new(file);
    let mut json = String::new();
    reader.read_to_string(&mut json).unwrap();

    let program: Program = serde_json::from_str(&json.as_str()).unwrap_or_else(|e| {
        println!("JSON manifest parsing error: {}", e);
        exit(-1);
    });

    program
}

fn run_program(args: &Vec<String>) {
    Program::load(&args[2]).run()
}
