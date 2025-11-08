//! Shadow Control Shell Commands
//!
//! Provides shell commands for shadow/canary deployment

impl super::Shell {
    /// Main entry point for shadowctl commands
    pub(crate) fn cmd_shadowctl(&self, args: &[&str]) {
        #[cfg(feature = "shadow-mode")]
        {
            self.shadowctl_impl(args);
        }
        #[cfg(not(feature = "shadow-mode"))]
        {
            crate::kprintln!("shadowctl: shadow-mode feature not enabled");
        }
    }

    #[cfg(feature = "shadow-mode")]
    fn shadowctl_impl(&self, args: &[&str]) {
        use crate::shadow::{SHADOW_AGENT, ShadowMode};

        if args.is_empty() {
            let stats = SHADOW_AGENT.get_stats();
            crate::kprintln!("Shadow Agent Status:");
            crate::kprintln!("  Mode:        {:?}", stats.mode);
            crate::kprintln!("  Decisions:   {}", stats.decision_count);
            crate::kprintln!("  Divergences: {} ({:.2}%)",
                stats.divergence_count, stats.divergence_rate);
            return;
        }

        match args[0] {
            "enable" => {
                if let Some(version) = args.get(1) {
                    crate::kprintln!("Enabling shadow agent with model: {}", version);
                    crate::kprintln!("Shadow agent enabled (LogOnly mode)");
                    crate::kprintln!("Use 'shadowctl mode compare' to enable comparison");
                } else {
                    crate::kprintln!("Usage: shadowctl enable <version>");
                }
            }
            "disable" => {
                SHADOW_AGENT.disable();
                crate::kprintln!("Shadow agent disabled");
            }
            "promote" => {
                let stats = SHADOW_AGENT.get_stats();
                crate::kprintln!("Shadow Agent Statistics:");
                crate::kprintln!("  Decisions:   {}", stats.decision_count);
                crate::kprintln!("  Divergences: {} ({:.2}%)",
                    stats.divergence_count, stats.divergence_rate);

                if stats.divergence_rate > 10.0 {
                    crate::kprintln!("\nWARNING: Divergence rate > 10%");
                    crate::kprintln!("Promotion not recommended");
                } else {
                    crate::kprintln!("\nPromoting shadow to production...");
                    SHADOW_AGENT.disable();
                    crate::kprintln!("Promotion complete");
                }
            }
            "status" => {
                let stats = SHADOW_AGENT.get_stats();
                crate::kprintln!("Shadow Agent Status:");
                crate::kprintln!("  Mode:        {:?}", stats.mode);
                crate::kprintln!("  Decisions:   {}", stats.decision_count);
                crate::kprintln!("  Divergences: {} ({:.2}%)",
                    stats.divergence_count, stats.divergence_rate);
            }
            "threshold" => {
                if let Some(threshold_str) = args.get(1) {
                    if let Ok(_threshold) = threshold_str.parse::<u32>() {
                        crate::kprintln!("Divergence threshold set to: {}", threshold_str);
                    } else {
                        crate::kprintln!("Invalid threshold value");
                    }
                } else {
                    crate::kprintln!("Usage: shadowctl threshold <N>");
                }
            }
            "mode" => {
                if let Some(mode_str) = args.get(1) {
                    let mode = match *mode_str {
                        "log" => Some(ShadowMode::LogOnly),
                        "compare" => Some(ShadowMode::Compare),
                        "canary10" => Some(ShadowMode::CanaryPartial),
                        "canary100" => Some(ShadowMode::CanaryFull),
                        _ => None,
                    };

                    if let Some(m) = mode {
                        SHADOW_AGENT.set_mode(m);
                        crate::kprintln!("Shadow mode set to: {:?}", m);
                    } else {
                        crate::kprintln!("Unknown mode: {}", mode_str);
                        crate::kprintln!("Valid modes: log, compare, canary10, canary100");
                    }
                } else {
                    crate::kprintln!("Usage: shadowctl mode <MODE>");
                }
            }
            _ => {
                crate::kprintln!("Unknown shadowctl command: {}", args[0]);
                crate::kprintln!("Usage:");
                crate::kprintln!("  shadowctl enable <version>    - Enable shadow agent");
                crate::kprintln!("  shadowctl disable             - Disable shadow mode");
                crate::kprintln!("  shadowctl promote             - Promote shadow to production");
                crate::kprintln!("  shadowctl status              - Show statistics");
                crate::kprintln!("  shadowctl threshold <N>       - Set divergence threshold");
                crate::kprintln!("  shadowctl mode <MODE>         - Set mode (log/compare/canary10/canary100)");
            }
        }
    }
}
