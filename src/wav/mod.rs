use std::{io, path::Path};

pub struct Wav {
}

impl Wav {
    pub fn empty() -> Self {
        unimplemented!();
    }

    pub fn read(_input_file: &Path) -> io::Result<Self> {
        unimplemented!();
    }

    pub fn write(&self, _output_file: &Path) -> io::Result<()> {
        unimplemented!();
    }
}
