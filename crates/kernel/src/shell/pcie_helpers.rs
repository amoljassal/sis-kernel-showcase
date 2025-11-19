//! Shell command helpers for PCIe control and diagnostics
//!
//! Provides interactive commands for testing and inspecting PCIe devices,
//! including the RP1 I/O Hub on Raspberry Pi 5.

use crate::drivers::pcie;

impl super::Shell {
    /// PCIe command handler
    ///
    /// Usage:
    ///   pcie                    - Show PCIe status
    ///   pcie scan [bus]         - Scan PCIe bus for devices (default: bus 0)
    ///   pcie info <bus> <dev> <func> - Show device information
    ///   pcie lspci              - List all PCIe devices (Linux-style)
    pub(crate) fn pcie_cmd(&self, args: &[&str]) {
        if !pcie::is_initialized() {
            unsafe {
                crate::uart_print(b"[PCIe] Not initialized\n");
            }
            return;
        }

        if args.is_empty() {
            self.pcie_status_cmd();
            return;
        }

        match args[0] {
            "scan" => {
                let bus = if args.len() >= 2 {
                    self.parse_number(args[1].as_bytes()).unwrap_or(0) as u8
                } else {
                    0
                };
                self.pcie_scan_cmd(bus);
            }
            "info" => {
                if args.len() < 4 {
                    unsafe {
                        crate::uart_print(b"Usage: pcie info <bus> <dev> <func>\n");
                        crate::uart_print(b"Example: pcie info 0 0 0\n");
                    }
                    return;
                }

                let bus = self.parse_number(args[1].as_bytes()).unwrap_or(0) as u8;
                let dev = self.parse_number(args[2].as_bytes()).unwrap_or(0) as u8;
                let func = self.parse_number(args[3].as_bytes()).unwrap_or(0) as u8;

                self.pcie_info_cmd(bus, dev, func);
            }
            "lspci" => {
                self.pcie_lspci_cmd();
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown pcie command: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Available commands: scan, info, lspci\n");
                }
            }
        }
    }

    /// Show PCIe subsystem status
    fn pcie_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== PCIe Subsystem Status ===\n");

            let _ = pcie::with_ecam(|ecam| {
                unsafe {
                    crate::uart_print(b"ECAM Base:  0x");
                    self.print_hex(ecam.base() as u64);
                    crate::uart_print(b"\n");

                    crate::uart_print(b"ECAM Size:  0x");
                    self.print_hex(ecam.size() as u64);
                    crate::uart_print(b"\n");
                }
                Ok(())
            });

            if let Some(rp1) = pcie::get_rp1() {
                crate::uart_print(b"\nRP1 I/O Hub: Present\n");
                crate::uart_print(b"  State:      ");
                match rp1.state() {
                    pcie::rp1::Rp1State::NotDetected => crate::uart_print(b"Not Detected\n"),
                    pcie::rp1::Rp1State::Detected => crate::uart_print(b"Detected\n"),
                    pcie::rp1::Rp1State::Initializing => crate::uart_print(b"Initializing\n"),
                    pcie::rp1::Rp1State::Ready => crate::uart_print(b"Ready\n"),
                    pcie::rp1::Rp1State::Error => crate::uart_print(b"Error\n"),
                }

                crate::uart_print(b"  Address:    ");
                let addr = rp1.pci_address();
                self.print_number_simple(addr.bus as u64);
                crate::uart_print(b":");
                self.print_number_simple(addr.device as u64);
                crate::uart_print(b".");
                self.print_number_simple(addr.function as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"  MMIO Base:  0x");
                self.print_hex(rp1.mmio_base() as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"  MMIO Size:  0x");
                self.print_hex(rp1.mmio_size() as u64);
                crate::uart_print(b"\n");
            } else {
                crate::uart_print(b"\nRP1 I/O Hub: Not Found\n");
            }

            crate::uart_print(b"\n");
        }
    }

    /// Scan PCIe bus for devices
    fn pcie_scan_cmd(&self, bus: u8) {
        unsafe {
            crate::uart_print(b"\n=== Scanning PCIe Bus ");
            self.print_number_simple(bus as u64);
            crate::uart_print(b" ===\n\n");
        }

        match pcie::scan_bus(bus) {
            Ok(devices) => {
                if devices.is_empty() {
                    unsafe {
                        crate::uart_print(b"No devices found\n");
                    }
                    return;
                }

                unsafe {
                    crate::uart_print(b"Found ");
                    self.print_number_simple(devices.len() as u64);
                    crate::uart_print(b" device(s):\n\n");
                }

                for dev in devices {
                    self.print_pcie_device(&dev);
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[PCIe] Error scanning bus\n");
                }
            }
        }
    }

    /// Show detailed device information
    fn pcie_info_cmd(&self, bus: u8, device: u8, function: u8) {
        let addr = pcie::ecam::PciAddress::new(bus, device, function);

        match pcie::get_device_info(addr) {
            Ok(dev) => {
                unsafe {
                    crate::uart_print(b"\n=== PCIe Device Information ===\n\n");
                }
                self.print_pcie_device_detailed(&dev);

                // Try to read BARs
                let _ = pcie::with_ecam(|ecam| {
                    unsafe {
                        crate::uart_print(b"\nBase Address Registers:\n");
                    }

                    for bar_idx in 0..6 {
                        match ecam.read_bar(addr, bar_idx) {
                            Ok(Some(bar)) => {
                                unsafe {
                                    crate::uart_print(b"  BAR");
                                    self.print_number_simple(bar.index as u64);
                                    crate::uart_print(b": 0x");
                                    self.print_hex(bar.base);
                                    crate::uart_print(b" (size: 0x");
                                    self.print_hex(bar.size);
                                    crate::uart_print(b") ");

                                    if bar.is_memory {
                                        crate::uart_print(b"[MEM");
                                        if bar.is_64bit {
                                            crate::uart_print(b" 64-bit");
                                        } else {
                                            crate::uart_print(b" 32-bit");
                                        }
                                        if bar.is_prefetchable {
                                            crate::uart_print(b" prefetchable");
                                        }
                                        crate::uart_print(b"]\n");
                                    } else {
                                        crate::uart_print(b"[I/O]\n");
                                    }
                                }

                                // Skip next BAR if this is a 64-bit BAR
                                if bar.is_64bit && bar_idx < 5 {
                                    break;
                                }
                            }
                            Ok(None) => {
                                // BAR not implemented, skip
                            }
                            Err(_) => {
                                unsafe {
                                    crate::uart_print(b"  BAR");
                                    self.print_number_simple(bar_idx as u64);
                                    crate::uart_print(b": <error reading>\n");
                                }
                            }
                        }
                    }
                    Ok(())
                });

                unsafe {
                    crate::uart_print(b"\n");
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[PCIe] Device not found or error reading info\n");
                }
            }
        }
    }

    /// List all PCIe devices (lspci-style output)
    fn pcie_lspci_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n");
        }

        match pcie::scan_bus(0) {
            Ok(devices) => {
                if devices.is_empty() {
                    unsafe {
                        crate::uart_print(b"No PCIe devices found\n");
                    }
                    return;
                }

                for dev in devices {
                    unsafe {
                        // Print BDF
                        if dev.address.bus < 10 {
                            crate::uart_print(b"0");
                        }
                        self.print_number_simple(dev.address.bus as u64);
                        crate::uart_print(b":");
                        if dev.address.device < 10 {
                            crate::uart_print(b"0");
                        }
                        self.print_number_simple(dev.address.device as u64);
                        crate::uart_print(b".");
                        self.print_number_simple(dev.address.function as u64);
                        crate::uart_print(b" ");

                        // Print class
                        self.print_device_class(dev.base_class(), dev.sub_class());
                        crate::uart_print(b": ");

                        // Print vendor:device
                        self.print_device_name(dev.vendor_id, dev.device_id);

                        crate::uart_print(b"\n");
                    }
                }
            }
            Err(_) => {
                unsafe {
                    crate::uart_print(b"[PCIe] Error scanning bus\n");
                }
            }
        }

        unsafe {
            crate::uart_print(b"\n");
        }
    }

    /// Print PCIe device summary
    fn print_pcie_device(&self, dev: &pcie::ecam::PciDevice) {
        unsafe {
            // BDF
            if dev.address.bus < 10 {
                crate::uart_print(b"0");
            }
            self.print_number_simple(dev.address.bus as u64);
            crate::uart_print(b":");
            if dev.address.device < 10 {
                crate::uart_print(b"0");
            }
            self.print_number_simple(dev.address.device as u64);
            crate::uart_print(b".");
            self.print_number_simple(dev.address.function as u64);
            crate::uart_print(b"  ");

            // Vendor:Device
            crate::uart_print(b"[");
            self.print_hex16(dev.vendor_id);
            crate::uart_print(b":");
            self.print_hex16(dev.device_id);
            crate::uart_print(b"]  ");

            // Class
            crate::uart_print(b"Class: ");
            self.print_hex16((dev.class_code >> 8) as u16);
            crate::uart_print(b"  ");

            self.print_device_name(dev.vendor_id, dev.device_id);

            crate::uart_print(b"\n");
        }
    }

    /// Print detailed PCIe device information
    fn print_pcie_device_detailed(&self, dev: &pcie::ecam::PciDevice) {
        unsafe {
            crate::uart_print(b"Address:        ");
            self.print_number_simple(dev.address.bus as u64);
            crate::uart_print(b":");
            self.print_number_simple(dev.address.device as u64);
            crate::uart_print(b".");
            self.print_number_simple(dev.address.function as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"Vendor ID:      0x");
            self.print_hex16(dev.vendor_id);
            crate::uart_print(b"\n");

            crate::uart_print(b"Device ID:      0x");
            self.print_hex16(dev.device_id);
            crate::uart_print(b"\n");

            crate::uart_print(b"Revision ID:    0x");
            self.print_hex8(dev.revision_id);
            crate::uart_print(b"\n");

            crate::uart_print(b"Class Code:     0x");
            self.print_hex(dev.class_code as u64);
            crate::uart_print(b" (");
            self.print_device_class(dev.base_class(), dev.sub_class());
            crate::uart_print(b")\n");

            if dev.subsystem_vendor != 0 {
                crate::uart_print(b"Subsystem:      [");
                self.print_hex16(dev.subsystem_vendor);
                crate::uart_print(b":");
                self.print_hex16(dev.subsystem_id);
                crate::uart_print(b"]\n");
            }
        }
    }

    /// Print device class name
    fn print_device_class(&self, base_class: u8, sub_class: u8) {
        let class_name = match (base_class, sub_class) {
            (0x00, _) => "Unclassified",
            (0x01, 0x00) => "SCSI controller",
            (0x01, 0x01) => "IDE controller",
            (0x01, 0x06) => "SATA controller",
            (0x01, 0x08) => "NVMe controller",
            (0x02, 0x00) => "Ethernet controller",
            (0x03, 0x00) => "VGA controller",
            (0x04, 0x00) => "Video controller",
            (0x04, 0x01) => "Audio controller",
            (0x05, 0x80) => "System peripheral",
            (0x06, 0x00) => "Host bridge",
            (0x06, 0x04) => "PCI bridge",
            (0x0C, 0x03) => "USB controller",
            _ => "Unknown",
        };

        unsafe {
            crate::uart_print(class_name.as_bytes());
        }
    }

    /// Print device vendor/device name
    fn print_device_name(&self, vendor_id: u16, device_id: u16) {
        let name = match (vendor_id, device_id) {
            (0x1DE4, 0x0001) => "Raspberry Pi RP1 I/O Hub",
            (0x14E4, _) => "Broadcom Device",
            (0x1B21, _) => "ASMedia Device",
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown Device");
                }
                return;
            }
        };

        unsafe {
            crate::uart_print(name.as_bytes());
        }
    }

    /// Print 8-bit hex value
    fn print_hex8(&self, val: u8) {
        let hex_chars = b"0123456789abcdef";
        unsafe {
            crate::uart_print(&[hex_chars[(val >> 4) as usize]]);
            crate::uart_print(&[hex_chars[(val & 0xF) as usize]]);
        }
    }

    /// Print 16-bit hex value
    fn print_hex16(&self, val: u16) {
        self.print_hex8((val >> 8) as u8);
        self.print_hex8(val as u8);
    }

    /// RP1 command handler
    ///
    /// Usage:
    ///   rp1               - Show RP1 status
    ///   rp1 status        - Show detailed RP1 status
    ///   rp1 peripherals   - List RP1 peripheral controllers
    pub(crate) fn rp1_cmd(&self, args: &[&str]) {
        if !pcie::is_initialized() {
            unsafe {
                crate::uart_print(b"[RP1] PCIe not initialized\n");
            }
            return;
        }

        let rp1 = match pcie::get_rp1() {
            Some(r) => r,
            None => {
                unsafe {
                    crate::uart_print(b"[RP1] RP1 I/O Hub not found\n");
                }
                return;
            }
        };

        if args.is_empty() || args[0] == "status" {
            self.rp1_status_cmd(rp1);
        } else if args[0] == "peripherals" {
            self.rp1_peripherals_cmd(rp1);
        } else {
            unsafe {
                crate::uart_print(b"Unknown rp1 command: ");
                crate::uart_print(args[0].as_bytes());
                crate::uart_print(b"\n");
                crate::uart_print(b"Available commands: status, peripherals\n");
            }
        }
    }

    /// Show RP1 status
    fn rp1_status_cmd(&self, rp1: &pcie::rp1::Rp1Driver) {
        unsafe {
            crate::uart_print(b"\n=== RP1 I/O Hub Status ===\n\n");

            crate::uart_print(b"State:          ");
            match rp1.state() {
                pcie::rp1::Rp1State::NotDetected => crate::uart_print(b"Not Detected\n"),
                pcie::rp1::Rp1State::Detected => crate::uart_print(b"Detected\n"),
                pcie::rp1::Rp1State::Initializing => crate::uart_print(b"Initializing\n"),
                pcie::rp1::Rp1State::Ready => crate::uart_print(b"Ready\n"),
                pcie::rp1::Rp1State::Error => crate::uart_print(b"Error\n"),
            }

            crate::uart_print(b"Initialized:    ");
            if rp1.is_initialized() {
                crate::uart_print(b"Yes\n");
            } else {
                crate::uart_print(b"No\n");
            }

            let addr = rp1.pci_address();
            crate::uart_print(b"PCIe Address:   ");
            self.print_number_simple(addr.bus as u64);
            crate::uart_print(b":");
            self.print_number_simple(addr.device as u64);
            crate::uart_print(b".");
            self.print_number_simple(addr.function as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"MMIO Base:      0x");
            self.print_hex(rp1.mmio_base() as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"MMIO Size:      0x");
            self.print_hex(rp1.mmio_size() as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"IRQ Status:     0x");
            self.print_hex(rp1.irq_status() as u64);
            crate::uart_print(b"\n\n");
        }
    }

    /// Show RP1 peripheral controllers
    fn rp1_peripherals_cmd(&self, rp1: &pcie::rp1::Rp1Driver) {
        unsafe {
            crate::uart_print(b"\n=== RP1 Peripheral Controllers ===\n\n");

            // I2C controllers
            crate::uart_print(b"I2C Controllers (");
            self.print_number_simple(pcie::rp1::RP1_I2C_COUNT as u64);
            crate::uart_print(b"):\n");
            for i in 0..pcie::rp1::RP1_I2C_COUNT {
                if let Some(base) = rp1.i2c_base(i) {
                    crate::uart_print(b"  I2C");
                    self.print_number_simple(i as u64);
                    crate::uart_print(b": 0x");
                    self.print_hex(base as u64);
                    crate::uart_print(b"\n");
                }
            }

            crate::uart_print(b"\nSPI Controllers (");
            self.print_number_simple(pcie::rp1::RP1_SPI_COUNT as u64);
            crate::uart_print(b"):\n");
            for i in 0..pcie::rp1::RP1_SPI_COUNT {
                if let Some(base) = rp1.spi_base(i) {
                    crate::uart_print(b"  SPI");
                    self.print_number_simple(i as u64);
                    crate::uart_print(b": 0x");
                    self.print_hex(base as u64);
                    crate::uart_print(b"\n");
                }
            }

            crate::uart_print(b"\nPWM Controllers (");
            self.print_number_simple(pcie::rp1::RP1_PWM_COUNT as u64);
            crate::uart_print(b"):\n");
            for i in 0..pcie::rp1::RP1_PWM_COUNT {
                if let Some(base) = rp1.pwm_base(i) {
                    crate::uart_print(b"  PWM");
                    self.print_number_simple(i as u64);
                    crate::uart_print(b": 0x");
                    self.print_hex(base as u64);
                    crate::uart_print(b"\n");
                }
            }

            crate::uart_print(b"\nGPIO Controller:\n");
            crate::uart_print(b"  GPIO: 0x");
            self.print_hex(rp1.gpio_base() as u64);
            crate::uart_print(b"\n\n");
        }
    }
}
