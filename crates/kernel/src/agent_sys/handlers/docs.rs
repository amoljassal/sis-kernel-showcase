//! Document operation handlers for AgentSys

use crate::control::CtrlError;
use crate::agent_sys::{protocol, AgentId, policy};
use crate::security::agent_policy::{Capability, Resource, PolicyDecision};
use crate::uart;
use crate::trace::metric_kv;

/// Simple document reference counter (Phase 1: in-memory only)
static mut NEXT_DOC_REF: u32 = 1;

/// Handle DOC_NEW (0x39): Create new document
pub fn handle_new(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_doc_new", 1);

    // Parse document name
    if payload.len() < 2 {
        return Err(CtrlError::BadFrame);
    }

    let name_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;
    if payload.len() < 2 + name_len {
        return Err(CtrlError::BadFrame);
    }

    let name_bytes = &payload[2..2 + name_len];
    let name = core::str::from_utf8(name_bytes).map_err(|_| CtrlError::BadFrame)?;

    let decision = policy().check(
        agent_id,
        Capability::DocBasic,
        &Resource::NoResource,
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    // Allocate doc ref
    let doc_ref = unsafe {
        let r = NEXT_DOC_REF;
        NEXT_DOC_REF += 1;
        r
    };

    uart::print_str("[AgentSys] DOC_NEW: name=");
    uart::print_str(name);
    uart::print_str(" ref=");
    uart::print_u32(doc_ref);
    uart::print_str("\n");
    uart::print_str("[DOC] Created\n");

    Ok(())
}

/// Handle DOC_EDIT (0x3A): Edit document
pub fn handle_edit(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_doc_edit", 1);

    if payload.len() < 6 {
        return Err(CtrlError::BadFrame);
    }

    let doc_ref = u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3]
    ]);
    let ops_count = u16::from_le_bytes([payload[4], payload[5]]);

    let decision = policy().check(
        agent_id,
        Capability::DocBasic,
        &Resource::DocRef(doc_ref),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] DOC_EDIT: ref=");
    uart::print_u32(doc_ref);
    uart::print_str(" ops=");
    uart::print_u16(ops_count);
    uart::print_str("\n");
    uart::print_str("[DOC] Edited\n");

    Ok(())
}

/// Handle DOC_SAVE (0x3B): Save document
pub fn handle_save(agent_id: AgentId, payload: &[u8]) -> Result<(), CtrlError> {
    metric_kv("agentsys_doc_save", 1);

    if payload.len() < 4 {
        return Err(CtrlError::BadFrame);
    }

    let doc_ref = u32::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3]
    ]);

    let decision = policy().check(
        agent_id,
        Capability::DocBasic,
        &Resource::DocRef(doc_ref),
    );

    if let PolicyDecision::Deny { reason } = decision {
        return Err(CtrlError::AuthFailed);
    }

    uart::print_str("[AgentSys] DOC_SAVE: ref=");
    uart::print_u32(doc_ref);
    uart::print_str("\n");
    uart::print_str("[DOC] Saved\n");

    Ok(())
}
