//! Automatic Rollback Logic
//!
//! Monitors shadow agent performance and triggers rollback
//! when divergence exceeds acceptable thresholds.

use alloc::string::String;
use super::agent::ShadowStats;

/// Rollback trigger configuration
pub struct RollbackTrigger {
    pub divergence_threshold: u32,       // Max divergences before rollback
    pub confidence_drop_threshold: u32,   // Max confidence drop
    pub error_rate_threshold: f32,        // Max error rate (%)
}

impl Default for RollbackTrigger {
    fn default() -> Self {
        Self {
            divergence_threshold: 50,
            confidence_drop_threshold: 300,  // 30%
            error_rate_threshold: 20.0,      // 20%
        }
    }
}

impl RollbackTrigger {
    /// Check if rollback should be triggered
    pub fn check(&self, stats: &ShadowStats) -> RollbackDecision {
        // 1. Check absolute divergence count
        if stats.divergence_count >= self.divergence_threshold as u64 {
            return RollbackDecision::Rollback {
                reason: String::from("Divergence threshold exceeded"),
                metric: alloc::format!("{} divergences", stats.divergence_count),
            };
        }

        // 2. Check divergence rate
        if stats.divergence_rate > self.error_rate_threshold {
            return RollbackDecision::Rollback {
                reason: String::from("Divergence rate too high"),
                metric: alloc::format!("{:.2}%", stats.divergence_rate),
            };
        }

        // 3. All checks passed
        RollbackDecision::Continue
    }
}

/// Rollback decision
#[derive(Debug)]
pub enum RollbackDecision {
    /// Continue with current deployment
    Continue,
    /// Rollback recommended
    Rollback {
        reason: String,
        metric: String,
    },
}

/// Execute automatic rollback if needed
pub fn auto_rollback_if_needed() -> crate::lib::error::Result<()> {
    use crate::shadow::SHADOW_AGENT;
    use crate::lib::error::Errno;

    let stats = SHADOW_AGENT.get_stats();
    let trigger = RollbackTrigger::default();

    match trigger.check(&stats) {
        RollbackDecision::Continue => Ok(()),
        RollbackDecision::Rollback { reason, metric } => {
            crate::kprintln!("[Shadow] Auto-rollback triggered: {} ({})", reason, metric);

            // Disable shadow
            SHADOW_AGENT.disable();

            // REAL MODEL LIFECYCLE ROLLBACK
            #[cfg(feature = "model-lifecycle")]
            {
                if let Some(lifecycle_mutex) = crate::model_lifecycle::get_model_lifecycle() {
                    if let Some(ref mut lifecycle) = *lifecycle_mutex.lock() {
                        // Get current shadow version before rollback
                        let shadow_version = lifecycle.get_shadow()
                            .map(|m| m.version.clone())
                            .unwrap_or_else(|| String::from("unknown"));

                        // Get stable version (active model)
                        let stable_version = lifecycle.get_active()
                            .map(|m| m.version.clone())
                            .unwrap_or_else(|| String::from("unknown"));

                        crate::uart::print_str("[Shadow] Rolling back: ");
                        crate::uart::print_str(&shadow_version);
                        crate::uart::print_str(" -> ");
                        crate::uart::print_str(&stable_version);
                        crate::uart::print_str("\n");

                        // Perform rollback
                        match lifecycle.rollback() {
                            Ok(_) => {
                                // Write rollback event to log
                                write_rollback_event(&shadow_version, &stable_version, &reason);

                                // Log to audit (if available)
                                #[cfg(feature = "agentsys")]
                                {
                                    crate::security::agent_audit::audit()
                                        .log_system_event("shadow_rollback", true);
                                }

                                crate::uart::print_str("[Shadow] Rollback complete\n");
                            }
                            Err(e) => {
                                crate::uart::print_str("[Shadow] Rollback failed: ");
                                crate::uart::print_str(e.description());
                                crate::uart::print_str("\n");
                                return Err(e);
                            }
                        }
                    }
                }
            }

            crate::kprintln!("[ROLLBACK] Complete - reverted to production model");

            Err(Errno::ECANCELED)
        }
    }
}

/// Write rollback event to JSON log
fn write_rollback_event(from: &str, to: &str, reason: &str) {
    use alloc::format;

    let timestamp = crate::time::get_uptime_ms();

    let mut json = String::from("{\"event\":\"rollback\",");
    json.push_str("\"from\":\"");
    json.push_str(from);
    json.push_str("\",\"to\":\"");
    json.push_str(to);
    json.push_str("\",\"time\":");
    json.push_str(&format!("{}", timestamp));
    json.push_str(",\"reason\":\"");
    json.push_str(reason);
    json.push_str("\"}\n");

    // Append to rollback log
    if let Ok(fd) = crate::vfs::open(
        "/var/log/rollback.json",
        crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_APPEND | crate::vfs::OpenFlags::O_CREAT
    ) {
        let _ = fd.write(json.as_bytes());
    } else {
        // Try to create directory first
        let _ = crate::vfs::mkdir("/var", 0o755);
        let _ = crate::vfs::mkdir("/var/log", 0o755);

        if let Ok(fd) = crate::vfs::open(
            "/var/log/rollback.json",
            crate::vfs::OpenFlags::O_WRONLY | crate::vfs::OpenFlags::O_APPEND | crate::vfs::OpenFlags::O_CREAT
        ) {
            let _ = fd.write(json.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow::agent::ShadowMode;

    #[test]
    fn test_rollback_trigger() {
        let trigger = RollbackTrigger::default();

        // Stats below threshold
        let stats = ShadowStats {
            mode: ShadowMode::Compare,
            decision_count: 100,
            divergence_count: 5,
            divergence_rate: 5.0,
        };

        match trigger.check(&stats) {
            RollbackDecision::Continue => {}
            _ => panic!("Should continue"),
        }

        // Stats above threshold
        let stats = ShadowStats {
            mode: ShadowMode::Compare,
            decision_count: 100,
            divergence_count: 60,
            divergence_rate: 60.0,
        };

        match trigger.check(&stats) {
            RollbackDecision::Rollback { .. } => {}
            _ => panic!("Should rollback"),
        }
    }
}
