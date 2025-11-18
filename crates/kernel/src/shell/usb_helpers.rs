//! Shell command helpers for USB device management
//!
//! Provides interactive commands for listing and managing USB devices.

use crate::drivers::usb;

impl super::Shell {
    /// USB command handler
    ///
    /// Usage:
    ///   usb                - Show USB status
    ///   usb list           - List connected USB devices
    ///   usb scan           - Scan for new devices
    ///   usb info <slot>    - Show detailed device info
    pub(crate) fn usb_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            self.usb_status_cmd();
            return;
        }

        match args[0] {
            "list" => {
                self.usb_list_cmd();
            }
            "scan" => {
                self.usb_scan_cmd();
            }
            "info" => {
                if args.len() < 2 {
                    unsafe {
                        crate::uart_print(b"Usage: usb info <slot_id>\n");
                        crate::uart_print(b"Example: usb info 1\n");
                    }
                    return;
                }

                let slot_id = self.parse_number(args[1].as_bytes()).unwrap_or(0) as u8;
                self.usb_info_cmd(slot_id);
            }
            "help" => {
                self.usb_help_cmd();
            }
            _ => {
                unsafe {
                    crate::uart_print(b"Unknown USB command: ");
                    crate::uart_print(args[0].as_bytes());
                    crate::uart_print(b"\n");
                    crate::uart_print(b"Try: usb help\n");
                }
            }
        }
    }

    /// Show USB subsystem status
    fn usb_status_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== USB Subsystem Status ===\n\n");
        }

        if usb::is_initialized() {
            unsafe {
                crate::uart_print(b"Status: Initialized and running\n");
            }

            // Get XHCI controller info
            if let Some(xhci) = usb::get_xhci() {
                unsafe {
                    crate::uart_print(b"Controller: XHCI (USB 3.0)\n");
                    crate::uart_print(b"State: ");
                    if xhci.is_running() {
                        crate::uart_print(b"Running\n");
                    } else {
                        crate::uart_print(b"Stopped\n");
                    }
                }
            }

            // Show device count
            let devices = usb::enumerate_devices();
            unsafe {
                crate::uart_print(b"Connected devices: ");
                self.print_number_simple(devices.len() as u64);
                crate::uart_print(b"\n");
            }
        } else {
            unsafe {
                crate::uart_print(b"Status: Not initialized\n");
                crate::uart_print(b"Note: USB requires RP1 PCIe driver\n");
            }
        }

        unsafe {
            crate::uart_print(b"\nUse 'usb list' to show connected devices\n");
        }
    }

    /// List connected USB devices
    fn usb_list_cmd(&self) {
        if !usb::is_initialized() {
            unsafe {
                crate::uart_print(b"[USB] Subsystem not initialized\n");
            }
            return;
        }

        unsafe {
            crate::uart_print(b"\n=== Connected USB Devices ===\n\n");
        }

        let devices = usb::enumerate_devices();

        if devices.is_empty() {
            unsafe {
                crate::uart_print(b"No USB devices found\n");
                crate::uart_print(b"Connect a USB device and run 'usb scan'\n");
            }
            return;
        }

        for device in &devices {
            unsafe {
                // Slot and Port
                crate::uart_print(b"Slot ");
                self.print_number_simple(device.id as u64);
                crate::uart_print(b" (Port ");
                self.print_number_simple(device.port as u64);
                crate::uart_print(b"):\n");

                // Device name
                crate::uart_print(b"  Name:     ");
                crate::uart_print(device.name.as_bytes());
                crate::uart_print(b"\n");

                // Vendor and Product ID
                crate::uart_print(b"  VID:PID:  ");
                self.print_usb_hex16(device.vendor_id);
                crate::uart_print(b":");
                self.print_usb_hex16(device.product_id);
                crate::uart_print(b"\n");

                // Class
                crate::uart_print(b"  Class:    ");
                crate::uart_print(device.class.name().as_bytes());
                crate::uart_print(b"\n");

                // Speed
                crate::uart_print(b"  Speed:    ");
                match device.speed {
                    usb::DeviceSpeed::Low => crate::uart_print(b"Low Speed (1.5 Mbps)\n"),
                    usb::DeviceSpeed::Full => crate::uart_print(b"Full Speed (12 Mbps)\n"),
                    usb::DeviceSpeed::High => crate::uart_print(b"High Speed (480 Mbps)\n"),
                    usb::DeviceSpeed::Super => crate::uart_print(b"SuperSpeed (5 Gbps)\n"),
                    usb::DeviceSpeed::SuperPlus => crate::uart_print(b"SuperSpeed+ (10 Gbps)\n"),
                }

                crate::uart_print(b"\n");
            }
        }

        unsafe {
            crate::uart_print(b"Total: ");
            self.print_number_simple(devices.len() as u64);
            crate::uart_print(b" device(s)\n");
        }
    }

    /// Scan for new USB devices
    fn usb_scan_cmd(&self) {
        if !usb::is_initialized() {
            unsafe {
                crate::uart_print(b"[USB] Subsystem not initialized\n");
            }
            return;
        }

        unsafe {
            crate::uart_print(b"[USB] Scanning for new devices...\n");
        }

        // Force re-enumeration
        let devices = usb::enumerate_devices();

        unsafe {
            crate::uart_print(b"[USB] Scan complete. Found ");
            self.print_number_simple(devices.len() as u64);
            crate::uart_print(b" device(s)\n");
        }

        if !devices.is_empty() {
            unsafe {
                crate::uart_print(b"Use 'usb list' to see details\n");
            }
        }
    }

    /// Show detailed device information
    fn usb_info_cmd(&self, slot_id: u8) {
        if !usb::is_initialized() {
            unsafe {
                crate::uart_print(b"[USB] Subsystem not initialized\n");
            }
            return;
        }

        let devices = usb::enumerate_devices();
        let device = devices.iter().find(|d| d.id == slot_id);

        if let Some(dev) = device {
            unsafe {
                crate::uart_print(b"\n=== USB Device Information ===\n\n");

                crate::uart_print(b"Slot ID:        ");
                self.print_number_simple(dev.id as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"Port:           ");
                self.print_number_simple(dev.port as u64);
                crate::uart_print(b"\n");

                crate::uart_print(b"Address:        ");
                self.print_number_simple(dev.address as u64);
                crate::uart_print(b"\n\n");

                crate::uart_print(b"Device Name:    ");
                crate::uart_print(dev.name.as_bytes());
                crate::uart_print(b"\n");

                if !dev.manufacturer.is_empty() {
                    crate::uart_print(b"Manufacturer:   ");
                    crate::uart_print(dev.manufacturer.as_bytes());
                    crate::uart_print(b"\n");
                }

                if !dev.product.is_empty() {
                    crate::uart_print(b"Product:        ");
                    crate::uart_print(dev.product.as_bytes());
                    crate::uart_print(b"\n");
                }

                if !dev.serial.is_empty() {
                    crate::uart_print(b"Serial:         ");
                    crate::uart_print(dev.serial.as_bytes());
                    crate::uart_print(b"\n");
                }

                crate::uart_print(b"\n");

                crate::uart_print(b"Vendor ID:      0x");
                self.print_usb_hex16(dev.vendor_id);
                crate::uart_print(b"\n");

                crate::uart_print(b"Product ID:     0x");
                self.print_usb_hex16(dev.product_id);
                crate::uart_print(b"\n");

                crate::uart_print(b"Class:          ");
                crate::uart_print(dev.class.name().as_bytes());
                crate::uart_print(b" (0x");
                self.print_usb_hex8(dev.class as u8);
                crate::uart_print(b")\n");

                crate::uart_print(b"Sub-Class:      0x");
                self.print_usb_hex8(dev.sub_class);
                crate::uart_print(b"\n");

                crate::uart_print(b"Protocol:       0x");
                self.print_usb_hex8(dev.protocol);
                crate::uart_print(b"\n");

                crate::uart_print(b"Speed:          ");
                match dev.speed {
                    usb::DeviceSpeed::Low => crate::uart_print(b"Low Speed (1.5 Mbps)\n"),
                    usb::DeviceSpeed::Full => crate::uart_print(b"Full Speed (12 Mbps)\n"),
                    usb::DeviceSpeed::High => crate::uart_print(b"High Speed (480 Mbps)\n"),
                    usb::DeviceSpeed::Super => crate::uart_print(b"SuperSpeed (5 Gbps)\n"),
                    usb::DeviceSpeed::SuperPlus => crate::uart_print(b"SuperSpeed+ (10 Gbps)\n"),
                }
            }
        } else {
            unsafe {
                crate::uart_print(b"[USB] Device slot ");
                self.print_number_simple(slot_id as u64);
                crate::uart_print(b" not found\n");
                crate::uart_print(b"Use 'usb list' to see available devices\n");
            }
        }
    }

    /// Show USB help
    fn usb_help_cmd(&self) {
        unsafe {
            crate::uart_print(b"\n=== USB Subsystem Commands ===\n\n");
            crate::uart_print(b"Commands:\n");
            crate::uart_print(b"  usb              - Show USB subsystem status\n");
            crate::uart_print(b"  usb list         - List all connected USB devices\n");
            crate::uart_print(b"  usb scan         - Scan for new USB devices\n");
            crate::uart_print(b"  usb info <slot>  - Show detailed device information\n");
            crate::uart_print(b"  usb help         - Show this help message\n\n");

            crate::uart_print(b"Device Classes:\n");
            crate::uart_print(b"  0x03 - HID (Human Interface Device)\n");
            crate::uart_print(b"  0x08 - Mass Storage (USB drives)\n");
            crate::uart_print(b"  0x0E - Video (Cameras, UVC)\n");
            crate::uart_print(b"  0x01 - Audio (Microphones, speakers)\n");
            crate::uart_print(b"  0x09 - Hub (USB hubs)\n\n");

            crate::uart_print(b"Examples:\n");
            crate::uart_print(b"  usb list         # List all devices\n");
            crate::uart_print(b"  usb info 1       # Show details for slot 1\n");
        }
    }

    /// Print 8-bit hex value (USB helper)
    fn print_usb_hex8(&self, val: u8) {
        let hex_chars = b"0123456789abcdef";
        unsafe {
            crate::uart_print(&[hex_chars[(val >> 4) as usize]]);
            crate::uart_print(&[hex_chars[(val & 0xF) as usize]]);
        }
    }

    /// Print 16-bit hex value (USB helper)
    fn print_usb_hex16(&self, val: u16) {
        self.print_usb_hex8((val >> 8) as u8);
        self.print_usb_hex8((val & 0xFF) as u8);
    }
}
