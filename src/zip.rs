use std::{fs::File, io, path::{Path, PathBuf}};

use zip::write::SimpleFileOptions;


pub enum ArchivedFile<'a> {
    File(&'a String),
    FileWithNewName(&'a String, &'a String),
}

pub struct Archiver<'a> {
    files: Vec<ArchivedFile<'a>>,
}

impl<'a> Archiver<'a> {
    pub fn new(files: Vec<ArchivedFile<'a>>) -> Self {
        Self { files }
    }

    pub fn compress(&self, archive_name: &str) {
        let mut archive = zip::ZipWriter::new(File::create(archive_name).unwrap());
        for file in self.files.iter() {
            
            let (path, name) = match file {
                ArchivedFile::File(path) => (Path::new(path), path.as_str()),
                ArchivedFile::FileWithNewName(path, name) => (Path::new(path), name.as_str())
            };
            
            let mut file = File::open(path).unwrap();
            let options = SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Bzip2).unix_permissions(0o755);
            archive.start_file(name, options).unwrap();
            io::copy(&mut file, &mut archive).unwrap();
        }
    }
}


pub struct Extractor {
    archive: PathBuf,
}

impl Extractor {
    pub fn new(archive: PathBuf) -> Self {
        Self { archive }
    }

    pub fn extract(&self, destination: PathBuf) {
        let mut archive = zip::ZipArchive::new(File::open(&self.archive).unwrap()).unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = destination.join(file.mangled_name());
            if (&*file.name()).ends_with('/') {
                std::fs::create_dir_all(&outpath).unwrap();
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(&p).unwrap();
                    }
                }
                std::fs::File::create(&outpath).unwrap();
            }
            
            {
                let mut outfile = std::fs::File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }
        }
    }
}