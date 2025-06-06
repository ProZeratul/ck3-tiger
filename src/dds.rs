//! Validator for the `.dds` (picture) files that are used in the game.

use std::fs::{metadata, File};
use std::io::{Read, Result};
use std::path::PathBuf;

use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::TigerHashMap;
use crate::parse::ParserMemory;
use crate::report::{err, tips, warn, ErrorKey};
#[cfg(feature = "ck3")]
use crate::token::Token;

const DDS_HEADER_SIZE: usize = 124;
const DDS_HEIGHT_OFFSET: usize = 12;
const DDS_WIDTH_OFFSET: usize = 16;
const DDS_PIXELFORMAT_FLAGS_OFFSET: usize = 80;

fn from_le32(buffer: &[u8], offset: usize) -> u32 {
    u32::from(buffer[offset])
        | (u32::from(buffer[offset + 1]) << 8)
        | (u32::from(buffer[offset + 2]) << 16)
        | (u32::from(buffer[offset + 3]) << 24)
}

#[derive(Clone, Debug, Default)]
pub struct DdsFiles {
    dds_files: TigerHashMap<String, DdsInfo>,
}

impl DdsFiles {
    fn load_dds(entry: &FileEntry) -> Result<Option<DdsInfo>> {
        if metadata(entry.fullpath())?.len() == 0 {
            warn(ErrorKey::ImageFormat).msg("empty file").loc(entry).push();
            return Ok(None);
        }
        let mut f = File::open(entry.fullpath())?;
        let mut buffer = [0; DDS_HEADER_SIZE];
        f.read_exact(&mut buffer)?;
        if buffer.starts_with(b"\x89PNG") {
            let msg = "actually a PNG";
            let info =
                "this may be slower and lower quality because PNG format does not have mipmaps";
            tips(ErrorKey::ImageFormat).msg(msg).info(info).loc(entry).push();
            return Ok(None);
        }
        if !buffer.starts_with(b"DDS ") {
            err(ErrorKey::ImageFormat).msg("not a DDS file").loc(entry).push();
            return Ok(None);
        }
        Ok(Some(DdsInfo::new(entry.clone(), &buffer)))
    }

    fn handle_dds(&mut self, entry: &FileEntry, info: DdsInfo) {
        self.dds_files.insert(entry.path().to_string_lossy().to_string(), info);
    }

    pub fn validate(&self) {
        for item in self.dds_files.values() {
            item.validate();
        }
    }

    #[cfg(feature = "ck3")]
    pub fn validate_frame(&self, key: &Token, width: u32, height: u32, frame: u32) {
        // Note: `frame` is 1-based
        if let Some(info) = self.dds_files.get(key.as_str()) {
            if info.height != height {
                let msg = format!("texture does not match frame height of {height}");
                warn(ErrorKey::ImageFormat).msg(msg).loc(key).push();
            } else if width == 0 || (info.width % width) != 0 {
                let msg = format!("texture is not a multiple of frame width {width}");
                warn(ErrorKey::ImageFormat).msg(msg).loc(key).push();
            } else if frame * width > info.width {
                let msg = format!("texture is not large enough for frame index {frame}");
                warn(ErrorKey::ImageFormat).msg(msg).loc(key).push();
            }
        }
    }
}

impl FileHandler<DdsInfo> for DdsFiles {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("gfx")
    }

    fn load_file(&self, entry: &FileEntry, _parser: &ParserMemory) -> Option<DdsInfo> {
        if !entry.filename().to_string_lossy().ends_with(".dds") {
            return None;
        }

        match Self::load_dds(entry) {
            Ok(info) => info,
            Err(e) => {
                err(ErrorKey::ReadError)
                    .msg("could not read dds header")
                    .info(format!("{e:#}"))
                    .loc(entry)
                    .push();
                None
            }
        }
    }

    fn handle_file(&mut self, entry: &FileEntry, info: DdsInfo) {
        self.handle_dds(entry, info);
    }
}

#[derive(Clone, Debug)]
pub struct DdsInfo {
    entry: FileEntry,
    compressed: bool,
    width: u32,
    height: u32,
}

impl DdsInfo {
    pub fn new(entry: FileEntry, header: &[u8]) -> Self {
        Self {
            entry,
            compressed: (from_le32(header, DDS_PIXELFORMAT_FLAGS_OFFSET) & 0x04) != 0,
            width: from_le32(header, DDS_WIDTH_OFFSET),
            height: from_le32(header, DDS_HEIGHT_OFFSET),
        }
    }

    fn validate(&self) {
        if self.compressed && !(self.width % 4 == 0 || self.height % 4 == 0) {
            let msg = "compressed DDS must have width and height divisible by 4";
            let info = format!(
                "DDS file is {}x{}, which can cause scaling problems and graphical artifacts",
                self.width, self.height
            );
            err(ErrorKey::ImageSize).msg(msg).info(info).loc(&self.entry).push();
        }
    }
}
