use std::{
    fs::File,
    ops::Not,
    path::{Path, PathBuf},
    process::Command,
    result,
};

use tempfile::{Builder, TempDir};

use crate::{
    filetransfer::{FilesTransfer, TransferStrategy},
    program::Manifest,
    zip::{ArchivedFile, Archiver, Extractor},
    ProgramResources,
};

pub struct Package {
    pub path: PathBuf,
}

type Error = String;
type Result = result::Result<(), Error>;

impl Package {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn unpack(&self) -> (UnpackedPackage, TempDir) {
        let dir = Builder::new().prefix("ebpm").tempdir_in("/var/tmp").unwrap();
        Extractor::new(self.path.clone()).extract(dir.path().to_path_buf());
        (dir.path().try_into().unwrap(), dir)
    }

    pub fn install(&self) -> Result {
        self.unpack().0.install(TransferStrategy::Move)
    }
}

pub struct UnpackedPackage {
    manifest: Manifest,
    path: PathBuf,
}

impl UnpackedPackage {
    pub fn install(&self, strategy: TransferStrategy) -> Result {
        std::env::set_current_dir(self.path.as_path()).unwrap();
        
        let resource = ProgramResources::new(&self.manifest.name);
        let transfer = FilesTransfer::new(strategy);

        transfer
            .execute_transfer(&self.manifest.files, &resource.res_path)
            .map_err(|err| format!("Failed to transfer files: {}", err))?;
           

        transfer
            .transfer_file("manifest.ebpm.json", &resource.manifest)
            .map_err(|err| err.to_string())?;

        if self.manifest.install_script.is_empty().not() {
            Command::new("sh")
                .current_dir(&resource.res_path)
                .arg(&self.manifest.install_script)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();
        };
        Ok(())
    }

    pub fn pack(&self) -> Package {
        std::env::set_current_dir(self.path.as_path()).unwrap();
        let manifest = String::from("manifest.ebpm.json");
        let files: Vec<ArchivedFile> = self
            .manifest
            .files
            .iter()
            .chain(std::iter::once(&manifest))
            .map(|file| ArchivedFile::File(file))
            .collect();

        let arvhiver = Archiver::new(files);
        let archive_name = format!("{}.ebpm.zip", self.manifest.name);
        arvhiver.compress(&archive_name);
        Package {
            path: PathBuf::from(archive_name),
        }
    }
}

impl TryFrom<&Path> for UnpackedPackage {
    type Error = Error;

    fn try_from(value: &Path) -> result::Result<Self, Self::Error> {
        let manifest: Manifest = File::open(value.join("manifest.ebpm.json"))
            .map_err(|err| format!("Failed to open manifest file: {}", err))?
            .try_into()
            .map_err(|err: serde_json::Error| format!("Failed to parse manifest file: {}", err))?;

        Ok(Self {
            path: value.into(),
            manifest,
        })
    }
}
