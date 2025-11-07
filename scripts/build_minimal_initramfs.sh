#!/bin/bash
# Minimal initramfs for Phase A1 testing (no BusyBox required)
#
# Creates a tiny root filesystem with just directories and a test file
# for validating initramfs unpacking.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$SCRIPT_DIR/.."
BUILD_DIR="$ROOT_DIR/build"
ROOTFS_DIR="$BUILD_DIR/rootfs_minimal"

echo "==> Building minimal test initramfs"

# Create root filesystem structure
rm -rf "$ROOTFS_DIR"
mkdir -p "$ROOTFS_DIR"/{bin,sbin,dev,proc,sys,tmp,etc}

# Create a simple test file
echo "Hello from initramfs!" > "$ROOTFS_DIR/test.txt"

# Create a simple init script (even though we can't exec it without BusyBox)
cat > "$ROOTFS_DIR/sbin/init" << 'EOF'
#!/bin/sh
echo "Minimal init - no shell available"
EOF
chmod 755 "$ROOTFS_DIR/sbin/init"

# Create basic /etc files
mkdir -p "$ROOTFS_DIR/etc"
echo "root:x:0:0:root:/root:/bin/sh" > "$ROOTFS_DIR/etc/passwd"
echo "root:x:0:" > "$ROOTFS_DIR/etc/group"

# Set permissions
chmod 755 "$ROOTFS_DIR"/{bin,sbin,dev,proc,sys,tmp,etc}
chmod 1777 "$ROOTFS_DIR/tmp"

# Create cpio archive
echo "==> Creating minimal cpio archive"
cd "$ROOTFS_DIR"
find . -print0 | cpio --null -o -H newc > "$BUILD_DIR/initramfs_minimal.cpio"
cd "$BUILD_DIR"

echo "==> initramfs_minimal.cpio created ($(du -h initramfs_minimal.cpio | cut -f1))"

# Verify
echo "==> Contents:"
cpio -t < initramfs_minimal.cpio

# Create Rust include file for minimal version
cat > "$ROOT_DIR/crates/kernel/initramfs_data_minimal.rs" << 'RUST_EOF'
// Minimal test initramfs data (no BusyBox)
pub const INITRAMFS_DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../build/initramfs_minimal.cpio"));
RUST_EOF

echo ""
echo "==> Minimal initramfs ready for testing"
echo "    File: $BUILD_DIR/initramfs_minimal.cpio"
