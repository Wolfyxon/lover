use std::path::Path;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use backhand::{FilesystemReader, InnerNode};

use crate::console::{exit_err, print_warn};
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

pub fn extract_squashfs_files(squashfs_path: &Path, output_path: &Path) {
    let file_reader = BufReader::new(files::open(squashfs_path));

    let reader = match FilesystemReader::from_reader(file_reader) {
        Ok(res) => res,
        Err(err) => exit_err(format!("Failed to read SquashFS: {}", err))
    };

    for node in reader.files() {
        let path = output_path.join(node.fullpath.strip_prefix("/").unwrap());
        
        match &node.inner {
            InnerNode::File(f) => {
                let file = files::create(path.as_path());
                
                let mut wr = BufWriter::with_capacity(f.basic.file_size as usize, &file);
                let mut rd = reader.file(&f.basic).reader();
                
                match std::io::copy(&mut rd, &mut wr) {
                    Ok(_) => {},
                    Err(err) => exit_err(format!("Extraction of '{}' failed: {}", path.to_str().unwrap(), err))
                };
            },
            InnerNode::Dir(_d) => {
                files::create_dir(path.as_path());
            },
            _ => {
                print_warn(format!("Unimplemented SquashFS node: {:?}", &node.inner));
            }
        }
    }
}