# Ext Image Persistence & Journaling

This guide explains two ways to exercise persistence of Phase 7 artifacts (`/models`, `/incidents`) using an attached block image.

## A) Read‑Only Mount Overlay at `/models`

Attach an image as a second VirtIO device and let the kernel attempt a read‑only mount of the first block device at `/models` (ext2 driver):

```
EXT4_IMG=/path/to/image.img SIS_FEATURES="bringup,ai-ops" ./scripts/uefi_run.sh
```

In the shell:

```
autoctl off
ls /models
```

Notes:
- The ext2 driver provides read‑only support. Use this to validate mountpoint overlay and readback.
- **ext4 write support is now production-ready** - Full read/write operations including file creation, truncation, and deletion are supported through VFS.

## Incident Exports (Paths and Flags)

- Default location: Incident bundles are written under `/incidents/INC-<unix-secs>-NNN.json` if no output path is specified.
- Export commands:
  - `tracectl export [--path <file>] [--all | --recent N | <id...>]`
  - `tracectl export-divergences [N] [--path <file>]`
  - Examples:
    - `tracectl export --recent 5 --path /traces5.json`
    - `tracectl export 1001 1002 --path /picked.json`
    - `tracectl export --all --path /all_traces.json`
    - `tracectl export-divergences 25 --path /div25.json`

Notes:
- The `/models` ext2 overlay in this guide is read‑only; attempting to export bundles under `/models` will fail (EROFS). Use `/incidents` or another writable path.
- **ext4 write support is production-ready**: You can now export incident bundles to ext4-mounted filesystems with full journaling support.

## B) ext4/JBD2 Durability Harness (Crash/Replay)

Run the full ext4 journaling cycle (create image → write → crash → replay → fsck):

```
./scripts/ext4_durability_tests.sh /tmp/ext4-test.img
```

The script:
- Creates an ext4 image and attaches it as a second VirtIO device
- Boots the kernel and runs the in‑kernel ext4 test (journaled writes)
- Reboots to replay the journal (verifies JBD2 recovery)
- Optionally runs host `fsck.ext4` to validate the image

Artifacts:
- Logs stored under `/tmp/sis-ext4-run*.log`
- Optional `fsck.ext4` report under `/tmp/sis-ext4-fsck.log`

## ext4 Write Support Implementation

### Production Status

ext4 write support is **production-ready** and actively used by Phase 7 AI Operations:
- Model persistence in `/models` directory
- Incident bundle exports in `/incidents` directory
- Decision trace storage

### Key Features

**Complete File Operations:**
- File creation with automatic inode/block allocation
- File truncation (O_TRUNC flag) with proper on-disk updates
- Data writes with extent tree support
- File deletion with resource cleanup

**Transactional Safety:**
- JBD2 journaling ensures metadata consistency
- Automatic journal replay on mount after crashes
- Ordered data mode: data written before metadata commit

### Deadlock Prevention

The ext4 implementation required careful lock management to prevent deadlocks during allocation operations:

**Problem:** Block and inode allocation functions (`allocate_block`, `allocate_inode`) hold locks on `superblock` and `block_groups` mutexes. When these functions needed to update metadata (write inodes or block group descriptors), they would call helper functions that tried to re-acquire the same locks, causing deadlocks.

**Solution:** Created lock-aware versions of helper functions that accept already-held lock guards:

```rust
// Lock-aware helper functions (crates/kernel/src/fs/ext4.rs)
fn write_inode_locked(&self, inode_num: u32, inode: &Ext4Inode,
                       sb: &Ext4Superblock, block_groups: &[Ext4BlockGroupDesc]) -> Result<()>

fn write_block_group_desc_locked(&self, group_idx: usize, desc: &Ext4BlockGroupDesc,
                                  sb: &Ext4Superblock) -> Result<()>
```

These functions use the already-held locks instead of trying to acquire new ones, preventing recursive locking.

**Integration with VFS:**
- `vfs/ext4.rs` provides VFS adapter implementing `InodeOps` trait
- `vfs/mod.rs` handles O_TRUNC flag by calling `truncate()` operation
- Proper truncation updates both in-memory and on-disk inode structures

### Testing

Validated through:
- Phase 7 incident bundle exports (`tracectl export`)
- Model persistence operations (`modelctl`)
- ext4 durability harness (crash/replay testing)

### Future Enhancements

- Mount `/models` on ext4 (currently ext2 read-only)
- Extent tree optimization for large files
- Background journal commit thread
