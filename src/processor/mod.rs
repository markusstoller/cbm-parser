mod archive;
mod file;

pub mod collector {
    pub(crate) use crate::processor::archive::{FileParser};
    pub(crate) use crate::processor::file::*;
}
