/// cpio newc format parser and unpacker
///
/// Parses ASCII hex cpio "newc" format archives and unpacks into tmpfs.

use crate::vfs::{S_IFDIR, S_IFREG};
use crate::lib::error::Errno;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;

/// cpio newc header (ASCII hex)
const NEWC_MAGIC: &[u8; 6] = b"070701";
const HEADER_SIZE: usize = 110; // newc header is 110 bytes
const TRAILER_NAME: &str = "TRAILER!!!";

/// Parse a 8-character hex string to u32
fn parse_hex(s: &[u8]) -> Result<u32, Errno> {
    if s.len() != 8 {
        return Err(Errno::EINVAL);
    }

    let mut result = 0u32;
    for &b in s {
        result = result << 4;
        result |= match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'f' => (b - b'a' + 10) as u32,
            b'A'..=b'F' => (b - b'A' + 10) as u32,
            _ => return Err(Errno::EINVAL),
        };
    }
    Ok(result)
}

/// Align offset to 4-byte boundary
fn align_4(offset: usize) -> usize {
    (offset + 3) & !3
}

/// cpio newc entry
struct CpioEntry {
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    nlink: u32,
    mtime: u32,
    filesize: u32,
    devmajor: u32,
    devminor: u32,
    rdevmajor: u32,
    rdevminor: u32,
    namesize: u32,
    check: u32,
}

impl CpioEntry {
    /// Parse header from bytes
    fn parse(data: &[u8]) -> Result<Self, Errno> {
        if data.len() < HEADER_SIZE {
            return Err(Errno::EINVAL);
        }

        // Check magic
        if &data[0..6] != NEWC_MAGIC {
            return Err(Errno::EINVAL);
        }

        Ok(Self {
            ino: parse_hex(&data[6..14])?,
            mode: parse_hex(&data[14..22])?,
            uid: parse_hex(&data[22..30])?,
            gid: parse_hex(&data[30..38])?,
            nlink: parse_hex(&data[38..46])?,
            mtime: parse_hex(&data[46..54])?,
            filesize: parse_hex(&data[54..62])?,
            devmajor: parse_hex(&data[62..70])?,
            devminor: parse_hex(&data[70..78])?,
            rdevmajor: parse_hex(&data[78..86])?,
            rdevminor: parse_hex(&data[86..94])?,
            namesize: parse_hex(&data[94..102])?,
            check: parse_hex(&data[102..110])?,
        })
    }

    /// Check if this is a directory
    fn is_dir(&self) -> bool {
        (self.mode & 0o170000) == 0o040000
    }

    /// Check if this is a regular file
    fn is_regular(&self) -> bool {
        (self.mode & 0o170000) == 0o100000
    }

    /// Get permission bits
    fn perm(&self) -> u32 {
        self.mode & 0o777
    }
}

/// Unpack cpio newc archive into tmpfs
pub fn unpack_initramfs(data: &[u8]) -> Result<(), Errno> {
    let root = crate::vfs::get_root().ok_or(Errno::ENOENT)?;

    let mut offset = 0;
    let mut entry_count = 0;

    crate::info!("initramfs: unpacking archive ({} bytes)", data.len());

    while offset < data.len() {
        // Parse header
        if offset + HEADER_SIZE > data.len() {
            break;
        }

        let entry = CpioEntry::parse(&data[offset..])?;
        offset += HEADER_SIZE;

        // Read filename
        if offset + entry.namesize as usize > data.len() {
            return Err(Errno::EINVAL);
        }

        let name_bytes = &data[offset..offset + entry.namesize as usize - 1]; // -1 to skip null terminator
        let name = core::str::from_utf8(name_bytes).map_err(|_| Errno::EINVAL)?;
        offset += entry.namesize as usize;
        offset = align_4(offset);

        // Check for trailer
        if name == TRAILER_NAME {
            crate::info!("initramfs: found trailer after {} entries", entry_count);
            break;
        }

        // Skip "." entry
        if name.is_empty() || name == "." {
            offset = align_4(offset + entry.filesize as usize);
            continue;
        }

        crate::debug!("initramfs: entry '{}' mode={:#o} size={}", name, entry.mode, entry.filesize);

        // Create entry based on type
        if entry.is_dir() {
            // Create directory
            create_directory(&root, name, entry.perm())?;
        } else if entry.is_regular() {
            // Create regular file
            if offset + entry.filesize as usize > data.len() {
                return Err(Errno::EINVAL);
            }

            let file_data = &data[offset..offset + entry.filesize as usize];
            create_file(&root, name, entry.perm(), file_data)?;

            offset += entry.filesize as usize;
        } else {
            // Skip other types (symlinks, devices, etc.) for Phase A1
            crate::debug!("initramfs: skipping special file '{}'", name);
            offset += entry.filesize as usize;
        }

        offset = align_4(offset);
        entry_count += 1;
    }

    crate::info!("initramfs: unpacked {} entries", entry_count);

    Ok(())
}

/// Create directory in tmpfs, creating parent directories as needed
fn create_directory(root: &Arc<crate::vfs::Inode>, path: &str, mode: u32) -> Result<(), Errno> {
    // Split path into components
    let components: Vec<&str> = path.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();

    if components.is_empty() {
        return Ok(()); // Root already exists
    }

    let mut current = root.clone();

    // Create each directory in the path
    for component in components {
        // Try to look up existing directory
        match current.lookup(component) {
            Ok(inode) => {
                current = inode;
            }
            Err(Errno::ENOENT) => {
                // Create new directory
                let new_dir = current.create(component, S_IFDIR | mode)?;
                current = new_dir;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Create file in tmpfs
fn create_file(root: &Arc<crate::vfs::Inode>, path: &str, mode: u32, data: &[u8]) -> Result<(), Errno> {
    // Split into parent directory and filename
    let (parent_path, filename) = match path.rfind('/') {
        Some(pos) => {
            let parent = if pos == 0 { "/" } else { &path[..pos] };
            let name = &path[pos + 1..];
            (parent, name)
        }
        None => ("/", path),
    };

    // Navigate to parent directory
    let parent = if parent_path == "/" {
        root.clone()
    } else {
        // Walk to parent
        let components: Vec<&str> = parent_path.trim_start_matches('/').split('/').filter(|s| !s.is_empty()).collect();
        let mut current = root.clone();

        for component in components {
            current = current.lookup(component)?;
        }

        current
    };

    // Create the file
    let file_inode = parent.create(filename, S_IFREG | mode)?;

    // Write content
    if !data.is_empty() {
        file_inode.write(0, data)?;
    }

    Ok(())
}
