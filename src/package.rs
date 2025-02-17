use std::{
    fs::File, ops::Not, path::{Path, PathBuf}, process::Command, result
};

use crate::{
    filetransfer::{FilesTransfer, TransferStrategy},
    program::Manifest,
    ProgramResources,
};

struct Package {
    path: PathBuf,
}

type Error = String;
type Result = result::Result<(), Error>;

impl Package {
    fn unpack(&self) -> UnpackedPackage {
        todo!("not implemented yet")
    }

    pub fn install(&self) -> Result {
        self.unpack().install(TransferStrategy::Copy)
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
            .map_err(|err| err.to_string()).unwrap();
        
        transfer.transfer_file("manifest.ebpm.json", &resource.manifest)
            .map_err(|err| err.to_string()).unwrap();
        
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
    
    
}

impl TryFrom<&Path> for UnpackedPackage {
    type Error = Error;

    fn try_from(value: &Path) -> result::Result<Self, Self::Error> {
        let manifest: Manifest = File::open(value.join("manifest.ebpm.json"))
            .map_err(|err| err.to_string())?
            .try_into()
            .map_err(|err: serde_json::Error| err.to_string())?;

        Ok(Self {
            path: value.into(),
            manifest,
        })
    }
}
