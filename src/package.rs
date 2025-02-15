use std::{
    fs::File,
    path::{Path, PathBuf},
    result,
};

use crate::{filetransfer::{FilesTransfer, TransferStrategy}, program::Manifest, ProgramResources};

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


struct UnpackedPackage {
    manifest: Manifest,
    path: PathBuf,
}

impl UnpackedPackage {
    pub fn install(&self, strategy: TransferStrategy) -> Result {
        let resource = ProgramResources::new(&self.manifest.name);
        let transfer = FilesTransfer::new(strategy);
        transfer.execute_transfer(&self.manifest.files, resource.res_path);
        todo!("not implemented yet")
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
