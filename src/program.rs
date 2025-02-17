use std::{
    collections::HashMap,
    env::{self},
    fs::{self, create_dir, File},
    io::{BufReader, Read, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{exit, Command},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub name: String,
    pub files: Vec<String>,
    pub install_script: String,
    remove_script: String,
    cmd: String,
}

impl Manifest {
    pub fn new(name: String, files: &Vec<String>) -> Self {
        Self {
            name,
            files: files.clone(),
            cmd: String::new(),
            install_script: String::new(),
            remove_script: String::new(),
        }
    }
}

impl TryFrom<File> for Manifest {
    type Error = serde_json::Error;

    fn try_from(value: File) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(value);
        let mut json = String::new();
        reader.read_to_string(&mut json).unwrap();
        serde_json::from_str(json.as_str())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Program {
    pub(crate) name: String,
    pub(crate) files: Vec<String>,
    cmd: String,
    install_script: String,
    remove_script: String,
}

impl Program {
    pub(crate) fn load(name: &str) -> Self {
        let path = std::env::home_dir()
            .unwrap()
            .join("Applications")
            .join(name.to_string() + ".json");

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

    pub(crate) fn run(&self) {
        let dir = ProgramResources::from(self);
        let mut vars: HashMap<String, String> = env::vars().collect();
        vars.insert(
            "RES".to_string(),
            dir.res_path.to_str().unwrap().to_string(),
        );

        let mut cmd = String::new();

        cmd.push_str(&self.cmd);
        cmd.push(' ');

        env::args().skip(3).for_each(|i| {
            cmd.push_str(i.replace("\\", "\\\\").replace("#", "\\#").as_str());
            cmd.push_str(" ");
        });

        Command::new("sh")
            .arg("-c")
            .env_clear()
            .envs(vars)
            .arg(cmd)
            .spawn()
            .unwrap_or_else(|er| {
                println!("Failed to run '{}' ", self.name);
                println!("Error: {}", er.to_string());
                exit(-1);
            })
            .wait()
            .unwrap();
    }

    pub(crate) fn remove(&self) {
        println!("Removing {}...", self.name);
        let dir = ProgramResources::from(self);
        if self.remove_script.len() > 0 {
            if let Err(er) = Command::new("bash")
                .current_dir(&dir.res_path)
                .arg(&self.remove_script)
                .spawn()
            {
                println!("Failed to run removing script'{}' ", self.remove_script);
                println!("Error: {}", er.to_string());
                exit(-1);
            }
        }

        fs::remove_file(dir.exe_path).unwrap();
        fs::remove_file(dir.manifest).unwrap();
        fs::remove_dir_all(dir.res_path).unwrap();
        println!("Removing finished");
    }
}

pub(crate) struct ProgramResources {
    pub(crate) res_path: PathBuf,
    pub(crate) exe_path: PathBuf,
    pub(crate) manifest: PathBuf,
}

impl ProgramResources {
    pub(crate) fn create_dir(path: &PathBuf) {
        match std::fs::metadata(path) {
            Err(_) => create_dir(path).unwrap(),
            Ok(e) => {
                e.is_file().then(|| {
                    println!(
                        "Error: cannot create '{}' because file with same name exist.",
                        path.to_str().unwrap()
                    );
                    exit(-1)
                });
            }
        };
    }

    pub fn new(name: &str) -> Self {
        let mut res = env::home_dir().unwrap();
        res.push("Applications");
        Self::create_dir(&res);
        let mut exe = res.clone();

        res.push("res");
        Self::create_dir(&res);
        res.push(name);
        Self::create_dir(&res);

        exe.push("exe");
        Self::create_dir(&exe);

        exe.push(name);

        let mut _file = match std::fs::metadata(exe.clone()) {
            Err(_) => {
                let mut file = File::create(exe.clone()).unwrap();
                file.write(b"#!/bin/bash\n").unwrap();
                file.write(b"ebpm run ").unwrap();
                file.write(name.as_bytes()).unwrap();
                file.write(b" $@").unwrap();
                fs::set_permissions(exe.clone(), fs::Permissions::from_mode(0o770)).unwrap();
            }
            Ok(e) => {
                e.is_dir().then(|| {
                    println!(
                        "Error: cannot create '{}' because directory with same name exist.",
                        exe.to_str().unwrap()
                    );
                    exit(-1)
                });
            }
        };

        let mut manifest = env::home_dir().unwrap();
        manifest.push("Applications");
        manifest.push(name.to_string() + ".json");
        ProgramResources {
            res_path: res,
            exe_path: exe,
            manifest,
        }
    }

    #[deprecated]
    pub(crate) fn from(program: &Program) -> Self {
        Self::new(&program.name)
    }
}
