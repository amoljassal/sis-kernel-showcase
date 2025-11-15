# Virtual File System (VFS) Subsystem

## What Lives Here

The VFS layer provides a unified abstraction for all filesystem operations in SIS Kernel. All filesystem implementations (procfs, tmpfs, ext2, ext4, devfs, ptsfs) integrate through a common trait-based interface.

**Core Components:**
- `mod.rs` - VFS core with mount table and global state
- `inode.rs` - Inode abstraction with `InodeOps` trait (the key integration point)
- `file.rs` - File descriptor table and open file management
- `mount.rs` - Mount point management

**Filesystem Implementations:**
- `procfs.rs` - `/proc` - System information (11 inodes: cpuinfo, meminfo, uptime, version, etc.)
- `tmpfs.rs` - RAM-backed temporary storage
- `ext2.rs` - Legacy ext2 support
- `devfs.rs` - `/dev` - Device nodes (character and block devices)
- `ptsfs.rs` - Pseudo-terminal filesystem
- `pipe.rs` - UNIX pipes for IPC

**Advanced (Phase F):**
- `../fs/ext4.rs` - Full ext4 with journaling (JBD2)
- `../fs/jbd2.rs` - Journal block device for crash recovery

## How to Extend: Adding a New Filesystem

All filesystems integrate through the **`InodeOps` trait** defined in `inode.rs`:

```rust
pub trait InodeOps: Send + Sync {
    fn read(&self, inode: &Inode, offset: usize, buf: &mut [u8]) -> Result<usize>;
    fn write(&self, inode: &Inode, offset: usize, buf: &[u8]) -> Result<usize>;
    fn readdir(&self, inode: &Inode, index: usize) -> Result<Option<DirEntry>>;
    fn lookup(&self, inode: &Inode, name: &str) -> Result<Option<Inode>>;
    fn create(&self, parent: &Inode, name: &str, mode: u16) -> Result<Inode>;
    fn mkdir(&self, parent: &Inode, name: &str, mode: u16) -> Result<Inode>;
    fn unlink(&self, parent: &Inode, name: &str) -> Result<()>;
    fn rmdir(&self, parent: &Inode, name: &str) -> Result<()>;
}
```

### Steps to Add a New Filesystem

1. **Create your filesystem module** (e.g., `myfs.rs`)
2. **Implement the `InodeOps` trait** for your filesystem operations
3. **Use `Box::leak()` for static lifetime** when creating trait objects:
   ```rust
   let ops: &'static dyn InodeOps = Box::leak(Box::new(MyFsOps));
   Inode::new(InodeType::Dir, mode, ops)
   ```
4. **Register mount points** in `mod.rs::init_vfs()`
5. **Add tests** to verify your filesystem operations

### Example: Minimal Read-Only Filesystem

```rust
pub struct ReadOnlyFs;

impl InodeOps for ReadOnlyFs {
    fn read(&self, inode: &Inode, offset: usize, buf: &mut [u8]) -> Result<usize> {
        // Your read implementation
        Ok(0)
    }

    fn write(&self, _inode: &Inode, _offset: usize, _buf: &[u8]) -> Result<usize> {
        Err(VfsError::ReadOnlyFs)  // Read-only
    }

    // Implement other required methods...
}
```

## Integration Points and Boundaries

### **Incoming:** What Depends on VFS
- **Syscall layer** (`../syscall/mod.rs`) - `sys_open`, `sys_read`, `sys_write`, `sys_close`
- **Process management** - File descriptor tables per process
- **Shell commands** - `cat`, `ls`, `mkdir` via syscall interface
- **Kernel initialization** - Mounts root filesystem at boot

### **Outgoing:** What VFS Depends On
- **Block devices** (`../drivers/virtio_blk.rs`) - For persistent filesystems (ext2, ext4)
- **Memory allocator** (`../heap.rs`) - For in-memory structures (inodes, file descriptors)
- **Security subsystem** (`../security/perm.rs`) - Permission checks via `check_permission()`

### **Boundary Rules**
1. **All filesystem access goes through VFS** - No direct access to filesystem implementations
2. **Path resolution is centralized** - `vfs::lookup_path()` handles all path traversal
3. **File descriptors are process-scoped** - Each process has independent fd table
4. **Mount points are global** - Shared mount table across all processes

## API Surface

### Core VFS Functions
```rust
// Path resolution
pub fn lookup_path(path: &str) -> Result<Inode>;

// File operations
pub fn vfs_open(path: &str, flags: u32, mode: u16) -> Result<usize>;  // Returns fd
pub fn vfs_read(fd: usize, buf: &mut [u8]) -> Result<usize>;
pub fn vfs_write(fd: usize, buf: &[u8]) -> Result<usize>;
pub fn vfs_close(fd: usize) -> Result<()>;

// Directory operations
pub fn vfs_mkdir(path: &str, mode: u16) -> Result<()>;
pub fn vfs_rmdir(path: &str) -> Result<()>;
pub fn vfs_readdir(fd: usize) -> Result<Vec<DirEntry>>;
```

### Inode Construction
```rust
// Standard pattern for creating inodes
let ops: &'static dyn InodeOps = Box::leak(Box::new(MyFsOps));
let inode = Inode::new(InodeType::RegularFile, 0o644, ops);
```

## Testing Your Filesystem

### Unit Tests
Add tests in your filesystem module:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write() {
        let ops = Box::leak(Box::new(MyFsOps));
        let inode = Inode::new(InodeType::RegularFile, 0o644, ops);

        // Test read/write operations
        let mut buf = [0u8; 256];
        assert!(ops.read(&inode, 0, &mut buf).is_ok());
    }
}
```

### Integration Tests
Test through syscall interface:
```rust
// In testing crate
let fd = vfs_open("/myfs/test.txt", O_RDWR | O_CREAT, 0o644)?;
vfs_write(fd, b"Hello, VFS!")?;
vfs_close(fd)?;
```

### Shell Testing
```bash
# From kernel shell
mkdir /myfs/testdir
echo "test" > /myfs/test.txt
cat /myfs/test.txt
ls /myfs
```

## Common Patterns and Gotchas

### ✅ Correct: Static Lifetime for Trait Objects
```rust
let ops: &'static dyn InodeOps = Box::leak(Box::new(MyFsOps));
```

### ❌ Incorrect: Non-Static Lifetime
```rust
let ops = MyFsOps;  // Won't compile - needs static lifetime
```

### ✅ Correct: Error Handling
```rust
fn read(&self, inode: &Inode, offset: usize, buf: &mut [u8]) -> Result<usize> {
    if offset > inode.size {
        return Err(VfsError::InvalidOffset);
    }
    // ...
}
```

### ❌ Incorrect: Panicking in VFS Operations
```rust
fn read(&self, inode: &Inode, offset: usize, buf: &mut [u8]) -> Result<usize> {
    assert!(offset < inode.size);  // DON'T PANIC - return error instead
    // ...
}
```

## Key Files and Lines of Code

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | ~300 | VFS core, mount table, global state |
| `inode.rs` | ~200 | Inode abstraction and `InodeOps` trait |
| `file.rs` | ~150 | File descriptor management |
| `procfs.rs` | ~250 | `/proc` filesystem (11 inodes) |
| `ext2.rs` | ~400 | ext2 filesystem support |
| `../fs/ext4.rs` | ~800 | ext4 with journaling |
| `../fs/jbd2.rs` | ~600 | Journal block device |

## Related Documentation

- **Phase A.1 VFS in main README** - High-level overview
- `../syscall/README.md` - Syscall integration
- `../fs/ext4.rs` - Advanced journaling implementation
- `docs/architecture/ARCHITECTURE.md` - System-wide architecture

## Performance Considerations

1. **File descriptor table lookups** - O(1) hash table access
2. **Path resolution** - O(n) where n = number of path components
3. **Directory listing** - O(m) where m = number of entries
4. **Inode caching** - Not yet implemented (future optimization)

## Future Work / TODOs

- [ ] Inode caching layer for frequently accessed files
- [ ] VFS statistics and metrics (file open/close counts, bytes transferred)
- [ ] Support for symbolic links and hard links
- [ ] Extended attributes (xattrs)
- [ ] File locking (flock, fcntl)
- [ ] Asynchronous I/O support
- [ ] Memory-mapped file support (mmap)

## Contact / Maintainers

VFS subsystem is core OS infrastructure. Changes should be reviewed carefully to avoid breaking filesystem implementations.

For questions about VFS architecture or adding new filesystems, refer to this README or the main project documentation.
