/// Partition table parsing (MBR and GPT)
///
/// Detects and parses partition tables to create BlockDevice entries
/// for each partition.

use crate::lib::error::{Result, Errno};
use crate::block::{BlockDevice, register_block_device};
use alloc::sync::Arc;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// MBR (Master Boot Record) signature
const MBR_SIGNATURE: u16 = 0xAA55;

/// GPT signature "EFI PART"
const GPT_SIGNATURE: [u8; 8] = *b"EFI PART";

/// Partition type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionType {
    Linux,
    LinuxSwap,
    LinuxLVM,
    EFISystem,
    Unknown(u8),
}

impl From<u8> for PartitionType {
    fn from(val: u8) -> Self {
        match val {
            0x83 => PartitionType::Linux,
            0x82 => PartitionType::LinuxSwap,
            0x8e => PartitionType::LinuxLVM,
            0xef => PartitionType::EFISystem,
            _ => PartitionType::Unknown(val),
        }
    }
}

/// Partition information
#[derive(Debug, Clone)]
pub struct Partition {
    /// Device name (e.g., "vda1", "vda2")
    pub name: String,
    /// Parent device name (e.g., "vda")
    pub parent: String,
    /// Partition number (1-based)
    pub number: u32,
    /// Starting sector (LBA)
    pub start_lba: u64,
    /// Number of sectors
    pub sector_count: u64,
    /// Partition type
    pub partition_type: PartitionType,
}

/// MBR partition table entry
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MbrPartitionEntry {
    status: u8,
    first_chs: [u8; 3],
    partition_type: u8,
    last_chs: [u8; 3],
    first_lba: u32,
    sector_count: u32,
}

/// MBR structure (sector 0)
#[repr(C, packed)]
struct Mbr {
    bootstrap: [u8; 446],
    partitions: [MbrPartitionEntry; 4],
    signature: u16,
}

/// GPT header
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GptHeader {
    signature: [u8; 8],
    revision: u32,
    header_size: u32,
    crc32: u32,
    reserved: u32,
    current_lba: u64,
    backup_lba: u64,
    first_usable_lba: u64,
    last_usable_lba: u64,
    disk_guid: [u8; 16],
    partition_entries_lba: u64,
    num_partition_entries: u32,
    partition_entry_size: u32,
    partition_entries_crc32: u32,
}

/// GPT partition entry
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GptPartitionEntry {
    type_guid: [u8; 16],
    partition_guid: [u8; 16],
    first_lba: u64,
    last_lba: u64,
    attributes: u64,
    name: [u16; 36], // UTF-16LE name
}

/// Probe and parse partition table on a block device
pub fn probe_partitions(device: &Arc<BlockDevice>) -> Result<Vec<Partition>> {
    // Read first sector (MBR or GPT protective MBR)
    let mut sector0 = vec![0u8; 512];
    device.read_sectors(0, &mut sector0)?;

    // Check for GPT first
    if let Ok(partitions) = try_parse_gpt(device, &sector0) {
        if !partitions.is_empty() {
            return Ok(partitions);
        }
    }

    // Fall back to MBR
    parse_mbr(device, &sector0)
}

/// Try to parse GPT partition table
fn try_parse_gpt(device: &Arc<BlockDevice>, mbr_sector: &[u8]) -> Result<Vec<Partition>> {
    // Read GPT header from LBA 1
    let mut gpt_header_sector = vec![0u8; 512];
    device.read_sectors(1, &mut gpt_header_sector)?;

    // Check GPT signature
    if &gpt_header_sector[0..8] != &GPT_SIGNATURE {
        return Err(Errno::EINVAL);
    }

    // Parse GPT header
    let header = unsafe {
        core::ptr::read_unaligned(gpt_header_sector.as_ptr() as *const GptHeader)
    };

    crate::info!("GPT: found {} partition entries at LBA {}",
                 header.num_partition_entries, header.partition_entries_lba);

    let mut partitions = Vec::new();

    // Read partition entries
    let entries_per_sector = 512 / header.partition_entry_size as usize;
    let sectors_needed = (header.num_partition_entries as usize + entries_per_sector - 1) / entries_per_sector;

    for sector_idx in 0..sectors_needed {
        let mut sector = vec![0u8; 512];
        device.read_sectors(header.partition_entries_lba + sector_idx as u64, &mut sector)?;

        for entry_idx in 0..entries_per_sector {
            let entry_offset = entry_idx * header.partition_entry_size as usize;
            if entry_offset + 128 > sector.len() {
                break;
            }

            let entry = unsafe {
                core::ptr::read_unaligned(
                    sector.as_ptr().add(entry_offset) as *const GptPartitionEntry
                )
            };

            // Check if entry is used (non-zero type GUID)
            if entry.type_guid.iter().all(|&b| b == 0) {
                continue;
            }

            let partition_num = (sector_idx * entries_per_sector + entry_idx + 1) as u32;
            let name = alloc::format!("{}{}", device.name, partition_num);

            partitions.push(Partition {
                name,
                parent: device.name.clone(),
                number: partition_num,
                start_lba: entry.first_lba,
                sector_count: entry.last_lba - entry.first_lba + 1,
                partition_type: PartitionType::Linux, // GPT uses GUIDs, default to Linux
            });
        }
    }

    Ok(partitions)
}

/// Parse MBR partition table
fn parse_mbr(device: &Arc<BlockDevice>, sector: &[u8]) -> Result<Vec<Partition>> {
    // Check MBR signature at offset 510-511
    let signature = u16::from_le_bytes([sector[510], sector[511]]);
    if signature != MBR_SIGNATURE {
        crate::warn!("MBR: invalid signature 0x{:04x}", signature);
        return Err(Errno::EINVAL);
    }

    let mut partitions = Vec::new();

    // Parse 4 primary partition entries (offset 446-509)
    for i in 0..4 {
        let entry_offset = 446 + (i * 16);
        let entry = unsafe {
            core::ptr::read_unaligned(
                sector.as_ptr().add(entry_offset) as *const MbrPartitionEntry
            )
        };

        // Skip empty partitions
        if entry.partition_type == 0 || entry.sector_count == 0 {
            continue;
        }

        let partition_num = (i + 1) as u32;
        let name = alloc::format!("{}{}", device.name, partition_num);

        crate::info!("MBR: partition {} type=0x{:02x} start={} count={}",
                     partition_num, entry.partition_type,
                     entry.first_lba, entry.sector_count);

        partitions.push(Partition {
            name,
            parent: device.name.clone(),
            number: partition_num,
            start_lba: entry.first_lba as u64,
            sector_count: entry.sector_count as u64,
            partition_type: PartitionType::from(entry.partition_type),
        });
    }

    if partitions.is_empty() {
        crate::info!("MBR: no valid partitions found");
    }

    Ok(partitions)
}

/// Register partitions as separate block devices
pub fn register_partitions(device: &Arc<BlockDevice>) -> Result<Vec<Arc<BlockDevice>>> {
    let partitions = probe_partitions(device)?;

    if partitions.is_empty() {
        crate::info!("partition: no partitions found on {}", device.name);
        return Ok(Vec::new());
    }

    let mut partition_devices = Vec::new();

    for partition in partitions {
        // Create a PartitionOps that wraps the parent device
        let part_dev = PartitionDevice::new(device.clone(), partition.clone());
        let part_ops: &'static PartitionOps = Box::leak(Box::new(PartitionOps {
            parent: device.clone(),
            start_lba: partition.start_lba,
            sector_count: partition.sector_count,
        }));

        let block_dev = BlockDevice::new(
            partition.name.clone(),
            device.major,
            partition.number,
            partition.sector_count,
            part_ops,
        );

        let dev = register_block_device(block_dev);
        partition_devices.push(dev);

        crate::info!("partition: registered {} ({} MB)",
                     partition.name,
                     partition.sector_count * 512 / 1024 / 1024);
    }

    Ok(partition_devices)
}

/// Partition device wrapper
struct PartitionDevice {
    parent: Arc<BlockDevice>,
    partition: Partition,
}

impl PartitionDevice {
    fn new(parent: Arc<BlockDevice>, partition: Partition) -> Self {
        Self { parent, partition }
    }
}

/// BlockDeviceOps for partitions (forwards to parent device with offset)
struct PartitionOps {
    parent: Arc<BlockDevice>,
    start_lba: u64,
    sector_count: u64,
}

impl crate::block::BlockDeviceOps for PartitionOps {
    fn read_sectors(&self, dev: &BlockDevice, sector: u64, buf: &mut [u8]) -> Result<()> {
        let sector_count = buf.len() / 512;
        if sector + sector_count as u64 > self.sector_count {
            return Err(Errno::EINVAL);
        }
        self.parent.read_sectors(self.start_lba + sector, buf)
    }

    fn write_sectors(&self, dev: &BlockDevice, sector: u64, buf: &[u8]) -> Result<()> {
        let sector_count = buf.len() / 512;
        if sector + sector_count as u64 > self.sector_count {
            return Err(Errno::EINVAL);
        }
        self.parent.write_sectors(self.start_lba + sector, buf)
    }

    fn flush(&self, dev: &BlockDevice) -> Result<()> {
        self.parent.flush()
    }
}
