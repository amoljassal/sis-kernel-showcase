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
        use crate::model_lifecycle::lifecycle::{get_model_lifecycle, ModelLifecycle};
        use crate::model_lifecycle::registry::ModelRegistry;
        use alloc::sync::Arc;
        use spin::Mutex;

        let ensure_lifecycle = || {
            let global = get_model_lifecycle().expect("global lifecycle mutex present");
            let need_init = {
                let guard = global.lock();
                guard.is_none()
            };
            if need_init {
                let reg = Arc::new(Mutex::new(ModelRegistry::new()));
                let _ = reg.lock().load();
                let mut guard = global.lock();
                if guard.is_none() {
                    *guard = Some(ModelLifecycle::new(reg));
                }
            }
            global
        };
        use crate::shadow::rollback::auto_rollback_if_needed;

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
            "dry-run" => {
                match args.get(1).copied() {
                    Some("on") => { SHADOW_AGENT.set_dry_run(true); crate::kprintln!("Shadow dry-run: ON"); }
                    Some("off") => { SHADOW_AGENT.set_dry_run(false); crate::kprintln!("Shadow dry-run: OFF"); }
                    Some("status") | None => {
                        crate::kprintln!("Shadow dry-run: {}", if SHADOW_AGENT.is_dry_run() { "ON" } else { "OFF" });
                    }
                    Some(_) => { crate::kprintln!("Usage: shadowctl dry-run on|off|status"); }
                }
            }
            "enable" => {
                if let Some(version) = args.get(1) {
                    crate::kprintln!("Enabling shadow agent with model: {}", version);
                    // Load shadow into lifecycle registry/state
                    let global = ensure_lifecycle();
                    let mut guard = global.lock();
                    if let Some(lc) = guard.as_mut() {
                        match lc.load_shadow(version) {
                            Ok(()) => {
                                if let Some(model) = lc.get_shadow() {
                                    let _ = SHADOW_AGENT.enable(model, ShadowMode::LogOnly);
                                    crate::kprintln!("Shadow agent enabled (LogOnly mode)");
                                    crate::kprintln!("Use 'shadowctl mode compare' to enable comparison");
                                } else {
                                    crate::kprintln!("enable: shadow model not present after load");
                                }
                            }
                            Err(e) => crate::kprintln!("enable: failed to load shadow (errno={:?})", e),
                        }
                    } else {
                        crate::kprintln!("shadowctl: lifecycle not initialized");
                    }
                } else {
                    crate::kprintln!("Usage: shadowctl enable <version>");
                }
            }
            // Back-compat alias for README examples
            "stats" => {
                let stats = SHADOW_AGENT.get_stats();
                crate::kprintln!("Shadow Agent Status:");
                crate::kprintln!("  Mode:        {:?}", stats.mode);
                crate::kprintln!("  Decisions:   {}", stats.decision_count);
                crate::kprintln!("  Divergences: {} ({:.2}%)",
                    stats.divergence_count, stats.divergence_rate);
            }
            // Canary routing percentage alias
            "canary" => {
                if let Some(pct) = args.get(1).and_then(|s| s.parse::<u32>().ok()) {
                    let mode = match pct {
                        10 => Some(ShadowMode::CanaryPartial),
                        100 => Some(ShadowMode::CanaryFull),
                        _ => None,
                    };
                    if let Some(m) = mode {
                        SHADOW_AGENT.set_mode(m);
                        crate::kprintln!("Shadow mode set to canary{}", pct);
                    } else {
                        crate::kprintln!("Invalid canary %. Use 10 or 100");
                    }
                } else {
                    crate::kprintln!("Usage: shadowctl canary <10|100>");
                }
            }
            // Manual rollback request alias
            "rollback" => {
                match auto_rollback_if_needed() {
                    Ok(()) => crate::kprintln!("Rollback not required (no trigger)"),
                    Err(_) => { /* messages printed by rollback */ }
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
                    if let Ok(threshold) = threshold_str.parse::<u32>() {
                        SHADOW_AGENT.set_threshold(threshold);
                        crate::kprintln!("Divergence threshold set to: {}", threshold);
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
