use std::path::Path;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::console::exit_err;
use crate::files;

// This is the offset I've seen in most AppImages (including LOVE)
// TODO: Implement automatic detection for the offset.
pub const SQUASHFS_OFFSET: u64 = 193728;

pub fn is_appimage(path: &Path) -> bool {
    let mut file = files::open(path);

    // AppImages always have 0x414902 at the offset of 8 bytes

    match file.seek(SeekFrom::Start(8)) {
        Ok(res) => res,
        Err(err) => exit_err(format!("Seek failed: {}", err))
    };

    let mut check_buffer = [0u8; 3];

    match file.read_exact(&mut check_buffer) {
        Ok(()) => {},
        Err(err) => exit_err(format!("Read failed: {}", err))
    };

    &check_buffer == b"\x41\x49\x02"
}

pub fn extract_squashfs(appimage_path: &Path, output_path: &Path) {
    if !is_appimage(appimage_path) {
        exit_err(format!("'{}' is not a valid AppImage", appimage_path.to_str().unwrap()));
    }
    
    let mut input_file = files::open(appimage_path);
    let mut output_file = files::create(output_path);

    match input_file.seek(SeekFrom::Start(SQUASHFS_OFFSET)) {
        Ok(res) => res,
        Err(err) => exit_err(format!("Seek failed: {}", err))
    };

    let mut buf: Vec<u8> = Vec::new();

    match input_file.read_to_end(&mut buf) {
        Ok(_) => {},
        Err(err) => exit_err(format!("Read failed: {}", err))
    };

    match output_file.write_all(&mut buf) {
        Ok(()) => {},
        Err(err) => exit_err(format!("Write failed: {}", err))
    };
}