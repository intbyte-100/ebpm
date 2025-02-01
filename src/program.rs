use std::{
    collections::HashMap,
    env::{self, args},
    fs::File,
    fs::{self, create_dir},
    io::{stdin, stdout, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{exit, Command},
};

use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::util::copy;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Program {
    pub(crate) name: String,
    pub(crate) files: Vec<String>,
    cmd: String,
    install_script: String,
    remove_script: String,
}

impl Program {
    pub(crate) fn new(name: String, files: &Vec<String>) -> Self {
        Program {
            name: name,
            files: files.clone(),
            cmd: String::new(),
            install_script: String::new(),
            remove_script: String::new(),
        }
    }

    pub(crate) fn install(&self) {
        println!("Installing {}...", &self.name);
        let dir = env::current_dir().unwrap();
        let resource = ProgramResources::from(self);
        let mut manifest = PathBuf::from(dir);
        manifest.push(self.name.clone() + ".json");
        Command::new("cp")
            .arg(manifest)
            .arg(resource.manifest)
            .spawn()
            .unwrap();

        if !self.files.is_empty() {
            println!("Copying files:");
            let error = self
                .files
                .iter()
                .map(|file| {
                    print!("{}", format!("    copying {}... ", file));
                    stdout().flush().unwrap();
                    match copy(file, &resource.res_path) {
                        Ok(_) => {
                            println!("[Success]");
                            false
                        }
                        Err(e) => {
                            println!("[Error: {}]", e.kind());
                            true
                        }
                    }
                })
                .find(|it| *it)
                .is_some();

            if error {
                println!("{}", "Error: instalation failed.".red());
                let program = (*self).clone();
                program.remove();
                exit(-1);
            }
        }

        if self.install_script.len() > 0 {
            Command::new("bash")
                .current_dir(&resource.res_path)
                .arg(&self.install_script)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        }
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

    pub(crate) fn from(program: &Program) -> Self {
        let mut res = env::home_dir().unwrap();
        res.push("Applications");
        Self::create_dir(&res);
        let mut exe = res.clone();

        res.push("res");
        Self::create_dir(&res);
        res.push(&program.name);
        Self::create_dir(&res);

        exe.push("exe");
        Self::create_dir(&exe);

        exe.push(&program.name);

        let mut _file = match std::fs::metadata(exe.clone()) {
            Err(_) => {
                let mut file = File::create(exe.clone()).unwrap();
                file.write(b"#!/bin/bash\n").unwrap();
                file.write(b"ebpm run ").unwrap();
                file.write(program.name.as_bytes()).unwrap();
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
        manifest.push(program.name.clone() + ".json");
        ProgramResources {
            res_path: res,
            exe_path: exe,
            manifest: manifest,
        }
    }
}
