// Helper for injecting control-plane frames as hex

impl super::Shell {
    pub(crate) fn ctlhex_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: ctlhex <hex>\n"); }
            return;
        }
        let s = args[0].trim();
        let bytes = s.as_bytes();
        let mut buf = [0u8; 256];
        let mut bi = 0usize;
        let mut i = 0usize;
        while i + 1 < bytes.len() && bi < buf.len() {
            let hn = match bytes[i] { b'0'..=b'9' => bytes[i] - b'0', b'a'..=b'f' => bytes[i] - b'a' + 10, b'A'..=b'F' => bytes[i] - b'A' + 10, _ => 0xFF };
            let ln = match bytes[i + 1] { b'0'..=b'9' => bytes[i + 1] - b'0', b'a'..=b'f' => bytes[i + 1] - b'a' + 10, b'A'..=b'F' => bytes[i + 1] - b'A' + 10, _ => 0xFF };
            if hn > 15 || ln > 15 { unsafe { crate::uart_print(b"[CTL] invalid hex\n"); } return; }
            buf[bi] = ((hn as u8) << 4) | (ln as u8);
            bi += 1; i += 2;
        }
        match crate::control::handle_frame(&buf[..bi]) {
            Ok(()) => unsafe { crate::uart_print(b"[CTL] ok\n"); }
            Err(_) => unsafe { crate::uart_print(b"[CTL] error\n"); }
        }
    }
}

