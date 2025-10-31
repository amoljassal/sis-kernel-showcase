use super::{GicDesc, MmioRange, Platform, RamRange, TimerDesc, UartDesc};

/// QEMU virt platform defaults
pub struct QemuVirtPlatform;

pub static INSTANCE: QemuVirtPlatform = QemuVirtPlatform;

impl Platform for QemuVirtPlatform {
    fn uart(&self) -> UartDesc {
        // QEMU virt PL011 UART at 0x0900_0000; UARTCLK ~= 24MHz
        UartDesc { base: 0x0900_0000, clock_hz: 24_000_000 }
    }

    fn gic(&self) -> GicDesc {
        // QEMU virt GICv3 distributor/redistributor bases
        GicDesc { gicd: 0x0800_0000, gicr: 0x080A_0000 }
    }

    fn timer(&self) -> TimerDesc {
        // QEMU generic timer frequency typically 62.5 MHz; runtime read of CNTFRQ_EL0 still preferred.
        TimerDesc { freq_hz: 62_500_000 }
    }

    fn mmio_ranges(&self) -> &'static [MmioRange] {
        const R: &[MmioRange] = &[
            MmioRange { start: 0x0800_0000, size: 0x0020_0000, device: true }, // GIC region
            MmioRange { start: 0x0900_0000, size: 0x0000_1000, device: true }, // PL011
            MmioRange { start: 0x0A00_0000, size: 0x0001_0000, device: true }, // VirtIO MMIO window (hint)
        ];
        R
    }

    fn ram_ranges(&self) -> &'static [RamRange] {
        // QEMU virt RAM default base 0x4000_0000 with -m controlling size; assume 512MiB in scripts.
        const R: &[RamRange] = &[
            RamRange { start: 0x4000_0000, size: 0x2000_0000 }, // 512 MiB
        ];
        R
    }

    fn virtio_mmio_hint(&self) -> Option<(usize, usize, u32)> {
        // QEMU virt: virtio-mmio window starts at 0x0A000000, slots are 0x200 bytes, IRQs start at 16
        Some((0x0A00_0000, 0x200, 16))
    }
}
