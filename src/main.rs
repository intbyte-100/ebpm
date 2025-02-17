mod filetransfer;
mod package;
mod program;
mod util;

use std::{
    env,
    fs::{self},
    io::Write,
    path::Path,
    process::{exit, Command},
};

use filetransfer::TransferStrategy;
use fs_extra::dir::get_size;
use package::UnpackedPackage;
use program::{Manifest, Program};
use util::request_yes_or_no;

use crate::program::ProgramResources;

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
        "list" => print_list(),

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

    let manifest = Manifest::new(args[2].clone(), &Vec::new());
    let json = serde_json::to_string_pretty(&manifest).unwrap();

    {
        std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("manifest.ebpm.json")
            .unwrap_or_else(|_e| fs::File::create("manifest.ebpm.json").unwrap())
            .write(json.as_str().as_bytes())
            .unwrap();
    }

    Command::new("micro")
        .arg("manifest.ebpm.json")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let install = request_yes_or_no("Do you want to install program?");

    if install {
        let strategy = match request_yes_or_no("Do you want to remove source package files?") {
            true => TransferStrategy::Move,
            false => TransferStrategy::Copy,
        };

        UnpackedPackage::try_from(env::current_dir().unwrap().as_path())
            .unwrap()
            .install(strategy)
            .unwrap()
    }
}

pub(crate) fn install_program(args: &Vec<String>) {
    let dir = env::current_dir().unwrap();

    if args.iter().filter(|it| it.as_str() == "-f").count() > 0 {
        args.iter()
            .skip(2)
            .filter(|t| t.as_str() != "-fa")
            .for_each(|it| {
                UnpackedPackage::try_from(Path::new(it))
                    .unwrap()
                    .install(TransferStrategy::Copy)
                    .unwrap();
            })
    }
}

fn remove_program(args: &Vec<String>) {
    Program::load(&args[2]).remove();
}

fn print_list() {
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

fn run_program(args: &Vec<String>) {
    Program::load(&args[2]).run()
}
