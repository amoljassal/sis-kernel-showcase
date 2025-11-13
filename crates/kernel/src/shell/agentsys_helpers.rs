//! Shell helpers for AgentSys testing and inspection

use crate::uart;
use crate::agent_sys::{self, AgentId};
use crate::security::agent_policy::AGENT_ID_TEST;

impl super::Shell {
    pub(crate) fn cmd_agentsys(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: agentsys <status|test-fs-list|test-audio-play|audit>\n"); }
            return;
        }
        match args[0] {
            "status" => { self.agentsys_status(); }
            "test-fs-list" => { self.agentsys_test_fs_list(); }
            "test-audio-play" => { self.agentsys_test_audio_play(); }
            "audit" => { self.agentsys_audit_dump(); }
            _ => unsafe { crate::uart_print(b"Usage: agentsys <status|test-fs-list|test-audio-play|audit>\n"); }
        }
    }

    fn agentsys_status(&self) {
        uart::print_str("[AgentSys] Status:\n");

        // Get policy engine stats
        let policy = agent_sys::policy();
        let agents = policy.list_agents();

        uart::print_str("  Registered agents: ");
        uart::print_u32(agents.len() as u32);
        uart::print_str("\n");

        for agent in agents {
            uart::print_str("    - ");
            uart::print_str(agent.name);
            uart::print_str(" (ID=");
            uart::print_u32(agent.agent_id);
            uart::print_str(", enabled=");
            uart::print_str(if agent.enabled { "yes" } else { "no" });
            uart::print_str(")\n");
        }

        // Get audit stats
        let audit = agent_sys::audit();
        uart::print_str("  Total operations: ");
        uart::print_u64(audit.total_operations());
        uart::print_str("\n");
    }

    fn agentsys_test_fs_list(&self) {
        uart::print_str("[AgentSys] Testing FS_LIST on /tmp/\n");

        // Build payload: path length (u16 LE) + path bytes
        let path = "/tmp/";
        let path_len = path.len() as u16;
        let mut payload = [0u8; 64];
        payload[0] = (path_len & 0xFF) as u8;
        payload[1] = ((path_len >> 8) & 0xFF) as u8;
        payload[2..2+path.len()].copy_from_slice(path.as_bytes());

        // Call handler directly (token with test agent ID)
        let token = (AGENT_ID_TEST as u64) << 48;
        let result = agent_sys::handle_frame(
            0x30, // FS_LIST
            token,
            &payload[0..2+path.len()]
        );

        match result {
            Ok(_) => uart::print_str("[AgentSys] Test PASSED\n"),
            Err(e) => {
                uart::print_str("[AgentSys] Test FAILED: ");
                uart::print_str(match e {
                    crate::control::CtrlError::AuthFailed => "AuthFailed",
                    crate::control::CtrlError::BadFrame => "BadFrame",
                    _ => "Unknown",
                });
                uart::print_str("\n");
            }
        }
    }

    fn agentsys_test_audio_play(&self) {
        uart::print_str("[AgentSys] Testing AUDIO_PLAY track=42\n");

        // Build payload: track_ref (u32 LE)
        let track_ref: u32 = 42;
        let mut payload = [0u8; 4];
        payload[0] = (track_ref & 0xFF) as u8;
        payload[1] = ((track_ref >> 8) & 0xFF) as u8;
        payload[2] = ((track_ref >> 16) & 0xFF) as u8;
        payload[3] = ((track_ref >> 24) & 0xFF) as u8;

        let token = (AGENT_ID_TEST as u64) << 48;
        let result = agent_sys::handle_frame(0x36, token, &payload);

        match result {
            Ok(_) => uart::print_str("[AgentSys] Test PASSED\n"),
            Err(_) => uart::print_str("[AgentSys] Test FAILED\n"),
        }
    }

    fn agentsys_audit_dump(&self) {
        uart::print_str("[AgentSys] Recent audit records:\n");
        let audit = agent_sys::audit();
        audit.dump_recent(10);
    }
}
