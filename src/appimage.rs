use std::path::Path;
use std::io::{Seek, SeekFrom, Read};
use crate::console::exit_err;
use crate::files;

pub fn is_appimage(appimage_path: &Path) -> bool {
    let mut file = files::open(appimage_path);

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