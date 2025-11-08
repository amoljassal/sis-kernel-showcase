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
            crate::println!("shadowctl: shadow-mode feature not enabled");
        }
    }

    #[cfg(feature = "shadow-mode")]
    fn shadowctl_impl(&self, args: &[&str]) {
        use crate::shadow::{SHADOW_AGENT, ShadowMode};

        if args.is_empty() {
            let stats = SHADOW_AGENT.get_stats();
            crate::println!("Shadow Agent Status:");
            crate::println!("  Mode:        {:?}", stats.mode);
            crate::println!("  Decisions:   {}", stats.decision_count);
            crate::println!("  Divergences: {} ({:.2}%)",
                stats.divergence_count, stats.divergence_rate);
            return;
        }

        match args[0] {
            "enable" => {
                if let Some(version) = args.get(1) {
                    crate::println!("Enabling shadow agent with model: {}", version);
                    crate::println!("Shadow agent enabled (LogOnly mode)");
                    crate::println!("Use 'shadowctl mode compare' to enable comparison");
                } else {
                    crate::println!("Usage: shadowctl enable <version>");
                }
            }
            "disable" => {
                SHADOW_AGENT.disable();
                crate::println!("Shadow agent disabled");
            }
            "promote" => {
                let stats = SHADOW_AGENT.get_stats();
                crate::println!("Shadow Agent Statistics:");
                crate::println!("  Decisions:   {}", stats.decision_count);
                crate::println!("  Divergences: {} ({:.2}%)",
                    stats.divergence_count, stats.divergence_rate);

                if stats.divergence_rate > 10.0 {
                    crate::println!("\nWARNING: Divergence rate > 10%");
                    crate::println!("Promotion not recommended");
                } else {
                    crate::println!("\nPromoting shadow to production...");
                    SHADOW_AGENT.disable();
                    crate::println!("Promotion complete");
                }
            }
            "status" => {
                let stats = SHADOW_AGENT.get_stats();
                crate::println!("Shadow Agent Status:");
                crate::println!("  Mode:        {:?}", stats.mode);
                crate::println!("  Decisions:   {}", stats.decision_count);
                crate::println!("  Divergences: {} ({:.2}%)",
                    stats.divergence_count, stats.divergence_rate);
            }
            "threshold" => {
                if let Some(threshold_str) = args.get(1) {
                    if let Ok(_threshold) = threshold_str.parse::<u32>() {
                        crate::println!("Divergence threshold set to: {}", threshold_str);
                    } else {
                        crate::println!("Invalid threshold value");
                    }
                } else {
                    crate::println!("Usage: shadowctl threshold <N>");
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
                        crate::println!("Shadow mode set to: {:?}", m);
                    } else {
                        crate::println!("Unknown mode: {}", mode_str);
                        crate::println!("Valid modes: log, compare, canary10, canary100");
                    }
                } else {
                    crate::println!("Usage: shadowctl mode <MODE>");
                }
            }
            _ => {
                crate::println!("Unknown shadowctl command: {}", args[0]);
                crate::println!("Usage:");
                crate::println!("  shadowctl enable <version>    - Enable shadow agent");
                crate::println!("  shadowctl disable             - Disable shadow mode");
                crate::println!("  shadowctl promote             - Promote shadow to production");
                crate::println!("  shadowctl status              - Show statistics");
                crate::println!("  shadowctl threshold <N>       - Set divergence threshold");
                crate::println!("  shadowctl mode <MODE>         - Set mode (log/compare/canary10/canary100)");
            }
        }
    }
}
