use std::{fs, io, path::Path};

#[derive(Copy, Clone)]
pub(crate) enum TransferStrategy {
    Move,
    Copy,
}

pub struct FilesTransfer {
    strategy: TransferStrategy,
}

impl FilesTransfer {
    pub fn transfer_file(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        match self.strategy {
            TransferStrategy::Move => fs::rename(src, dst)?,
            TransferStrategy::Copy => {
                fs::copy(src, dst)?;
            }
        };

        Ok(())
    }

    fn recoursive_copy(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        fs::create_dir_all(&dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;

            let ty = entry.file_type()?;
            if ty.is_dir() {
                self.recoursive_copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
            } else {
                self.transfer_file(entry.path(), dst.as_ref().join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    fn copy(&self, src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
        if let Ok(file) = fs::metadata(&src) {
            if file.is_file() {
                let file = src.as_ref().file_name().unwrap();
                self.transfer_file(src.as_ref(), dst.as_ref().join(file))?;
                return Ok(());
            }
        }

        let dir = src
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        self.recoursive_copy(src, dst.as_ref().join(dir))
    }

    pub fn execute_transfer(&self, files: &Vec<String>, dst: impl AsRef<Path>) -> io::Result<()> {
        for file in files.iter() {
            self.copy(file, dst.as_ref())?
        }
        Ok(())
    }

    pub fn new(strategy: TransferStrategy) -> Self {
        Self { strategy }
    }
}
