use std::path::Path;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use backhand::{FilesystemReader, InnerNode, SquashfsFileReader};

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

pub fn read_squashfs(path: &Path) -> FilesystemReader<'_> {
    let file_reader = BufReader::new(files::open(path));

    match FilesystemReader::from_reader(file_reader) {
        Ok(res) => res,
        Err(err) => exit_err(format!("Failed to read SquashFS: {}", err))
    }
}

pub fn write_from_squashfs_file(reader: &FilesystemReader<'_>, squashfs_file: &SquashfsFileReader, output_path: &Path) {
    let file = files::create(output_path);
    
    let mut wr = BufWriter::with_capacity(squashfs_file.basic.file_size as usize, &file);
    let mut rd = reader.file(&squashfs_file.basic).reader();
    
    match std::io::copy(&mut rd, &mut wr) {
        Ok(_) => {},
        Err(err) => exit_err(format!("Extraction to '{}' failed: {}", output_path.to_str().unwrap(), err))
    };
}

pub fn extract_squashfs_files(squashfs_path: &Path, output_path: &Path) {
    let reader = read_squashfs(squashfs_path);

    for node in reader.files() {
        let path = output_path.join(node.fullpath.strip_prefix("/").unwrap());
        
        match &node.inner {
            InnerNode::File(f) => {
                write_from_squashfs_file(&reader, f, &path);
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

pub fn extract_squashfs_file(squashfs_path: &Path, file_path: &Path, output_path: &Path) {
    let reader = read_squashfs(squashfs_path);

    for node in reader.files() {
        let path = output_path.join(node.fullpath.strip_prefix("/").unwrap());
        
        if path != file_path {
            continue;
        }

        match &node.inner {
            InnerNode::File(f) => {
                write_from_squashfs_file(&reader, f, output_path);
            },
            _ => exit_err(format!("'{}' is not a file.", file_path.to_str().unwrap()))
        };
    }

    exit_err(format!("File '{}' not found in SquashFS", file_path.to_str().unwrap()));
}