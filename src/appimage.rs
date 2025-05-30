use std::path::Path;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use backhand::{FilesystemReader, FilesystemWriter, InnerNode, SquashfsFileReader};

use crate::console::exit_err;
use crate::files;

// This is the offset I've seen in most AppImages (including LOVE)
// TODO: Implement automatic detection for the offset.
pub const SQUASHFS_OFFSET: u64 = 193728;

pub fn is_appimage(path: &Path) -> bool {
    let mut file = files::open(path);

    // AppImages always have 0x414902 at the offset of 8 bytes

    file.seek(SeekFrom::Start(8)).unwrap_or_else(|err| {
        exit_err(format!("Seek failed: {}", err))
    });

    let mut check_buffer = [0u8; 3];

    file.read_exact(&mut check_buffer).unwrap_or_else(|err| {
        exit_err(format!("Read failed: {}", err))
    });

    &check_buffer == b"\x41\x49\x02"
}

pub fn extract_squashfs(appimage_path: &Path, output_path: &Path) {
    if !is_appimage(appimage_path) {
        exit_err(format!("'{}' is not a valid AppImage", appimage_path.to_str().unwrap()));
    }
    
    let mut input_file = files::open(appimage_path);
    let mut output_file = files::create(output_path);

    input_file.seek(SeekFrom::Start(SQUASHFS_OFFSET)).unwrap_or_else(|err| {
        exit_err(format!("Seek failed: {}", err));
    });

    let mut buf: Vec<u8> = Vec::new();

    input_file.read_to_end(&mut buf).unwrap_or_else(|err| {
        exit_err(format!("Read failed: {}", err));
    });

    output_file.write_all(&mut buf).unwrap_or_else(|err| {
        exit_err(format!("Write failed: {}", err));
    });
}

pub fn embed_squashfs(appimage_path: &Path, squashfs_path: &Path) {
    if !is_appimage(appimage_path) {
        exit_err(format!("'{}' is not a valid AppImage", appimage_path.to_str().unwrap()));
    }

    let mut appimage = files::open_rw(appimage_path);
    let mut squashfs = files::open(squashfs_path);

    appimage.seek(SeekFrom::Start(SQUASHFS_OFFSET)).unwrap_or_else(|err| {
        exit_err(format!("Seek failed: {}", err));
    });

    std::io::copy(&mut squashfs, &mut appimage).unwrap_or_else(|err| {
        exit_err(format!("Failed to write SquashFS into AppImage: {}", err));
    });
}

pub fn read_squashfs(path: &Path) -> FilesystemReader<'_> {
    let file_reader = BufReader::new(files::open(path));

    FilesystemReader::from_reader(file_reader).unwrap_or_else(|err| {
        exit_err(format!("Failed to read SquashFS: {}", err));
    })
}

pub fn write_from_squashfs_file(reader: &FilesystemReader<'_>, squashfs_file: &SquashfsFileReader, output_path: &Path) {
    let file = files::create(output_path);
    
    let mut wr = BufWriter::with_capacity(squashfs_file.file_len(), &file);
    let mut rd = reader.file(&squashfs_file).reader();
    
    std::io::copy(&mut rd, &mut wr).unwrap_or_else(|err| {
        exit_err(format!("Extraction to '{}' failed: {}", output_path.to_str().unwrap(), err));
    });
}

pub fn extract_squashfs_file(squashfs_path: &Path, file_path: &Path, output_path: &Path) {
    let reader = read_squashfs(squashfs_path);

    for node in reader.files() {
        match &node.inner {
            InnerNode::File(f) => {
                if !files::compare_paths(file_path, node.fullpath.as_path()) {
                    continue;
                }

                write_from_squashfs_file(&reader, f, output_path);
                return;
            },
            _ => {}
        };
    }

    exit_err(format!("File '{}' not found in SquashFS", file_path.to_str().unwrap()));
}

pub fn replace_file_in_squashfs(squashfs_path: &Path, file_path: &Path, inner_path: &Path, new_squashfs_path: &Path) {
    let file = files::open_rw(file_path);

    let sfs_reader = read_squashfs(squashfs_path);

    let mut sfs_writer = FilesystemWriter::from_fs_reader(&sfs_reader).unwrap_or_else(|err| {
        exit_err(format!("Failed to initialize writer: {}", err));
    });

    sfs_writer.replace_file(inner_path, file).unwrap_or_else(|err| {
        exit_err(format!("Failed to write into SquashFS: {}", err));
    });

    let sfs_file = files::create(&new_squashfs_path);
    
    sfs_writer.write(sfs_file).unwrap_or_else(|err| {
        exit_err(format!("Failed to save new SquashFS: {}", err))
    });
}