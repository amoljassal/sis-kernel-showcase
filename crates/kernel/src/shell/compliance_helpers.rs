// Helpers for compliance commands (eu-ai-act/audit/transparency/checklist/incidents)

impl super::Shell {
    /// Show EU AI Act compliance report
    pub(crate) fn compliance_eu_ai_act(&self, _args: &[&str]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"EU AI ACT COMPLIANCE REPORT\n");
            crate::uart_print(b"========================================\n\n");
        }

        let compliance = crate::compliance::EU_AI_ACT_COMPLIANCE.lock();
        let score = compliance.compliance_score();

        unsafe {
            crate::uart_print(b"Article 13: Transparency\n");
            crate::uart_print(b"  Decision rationale: ");
            if compliance.decision_rationale_available {
                crate::uart_print(b"[OK] Available\n");
            } else {
                crate::uart_print(b"[X] Missing\n");
            }

            crate::uart_print(b"  Explanations: ");
            if compliance.explanation_provided {
                crate::uart_print(b"[OK] Provided\n");
            } else {
                crate::uart_print(b"[X] Not provided\n");
            }

            crate::uart_print(b"  Human-readable output: ");
            if compliance.human_readable_output {
                crate::uart_print(b"[OK] Yes\n\n");
            } else {
                crate::uart_print(b"[X] No\n\n");
            }

            crate::uart_print(b"Article 14: Human Oversight\n");
            crate::uart_print(b"  Override available: ");
            if compliance.human_override_available {
                crate::uart_print(b"[OK] Yes\n");
            } else {
                crate::uart_print(b"[X] No\n");
            }

            crate::uart_print(b"  Approval workflows: ");
            if compliance.approval_workflow_enabled {
                crate::uart_print(b"[OK] Enabled\n");
            } else {
                crate::uart_print(b"[ ] Optional\n");
            }

            crate::uart_print(b"  Stop mechanism: ");
            if compliance.stop_mechanism_functional {
                crate::uart_print(b"[OK] Functional\n\n");
            } else {
                crate::uart_print(b"[X] Not functional\n\n");
            }

            crate::uart_print(b"Article 15: Accuracy & Robustness\n");
            crate::uart_print(b"  Accuracy certified: ");
            if compliance.accuracy_certified {
                crate::uart_print(b"[OK] Yes\n");
            } else {
                crate::uart_print(b"[X] No\n");
            }

            crate::uart_print(b"  Robustness tested: ");
            if compliance.robustness_tested {
                crate::uart_print(b"[OK] Yes\n");
            } else {
                crate::uart_print(b"[X] No\n");
            }

            crate::uart_print(b"  OOD detection: ");
            if compliance.ood_detection_enabled {
                crate::uart_print(b"[OK] Enabled\n");
            } else {
                crate::uart_print(b"[X] Disabled\n");
            }

            crate::uart_print(b"  Adversarial testing: ");
            if compliance.adversarial_testing_passed {
                crate::uart_print(b"[OK] Passed\n");
            } else {
                crate::uart_print(b"[X] Failed\n");
            }

            crate::uart_print(b"  Cybersecurity: ");
            if compliance.cybersecurity_validated {
                crate::uart_print(b"[OK] Validated\n\n");
            } else {
                crate::uart_print(b"[X] Not validated\n\n");
            }

            crate::uart_print(b"Article 16: Recordkeeping\n");
            crate::uart_print(b"  Automatic logging: ");
            if compliance.automatic_logging_enabled {
                crate::uart_print(b"[OK] Enabled\n");
            } else {
                crate::uart_print(b"[X] Disabled\n");
            }

            crate::uart_print(b"  Audit trail: ");
            if compliance.audit_trail_complete {
                crate::uart_print(b"[OK] Complete\n");
            } else {
                crate::uart_print(b"[X] Incomplete\n");
            }

            crate::uart_print(b"  Retention: ");
            self.print_number_simple(compliance.retention_period_days as u64);
            crate::uart_print(b" days\n\n");

            crate::uart_print(b"========================================\n");
            crate::uart_print(b"OVERALL COMPLIANCE SCORE: ");
            self.print_number_simple(score as u64);
            crate::uart_print(b"%\n");

            if score >= 90 {
                crate::uart_print(b"Status: [OK] COMPLIANT (High-Risk AI System)\n");
            } else if score >= 70 {
                crate::uart_print(b"Status: [WARN] PARTIALLY COMPLIANT\n");
            } else {
                crate::uart_print(b"Status: [X] NON-COMPLIANT\n");
            }
            crate::uart_print(b"========================================\n\n");
        }

        drop(compliance);
    }

    /// Generate third-party audit package
    pub(crate) fn compliance_audit(&self, _args: &[&str]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"THIRD-PARTY AUDIT PACKAGE\n");
            crate::uart_print(b"========================================\n\n");
        }

        let mut audit = crate::compliance::AUDIT_METRICS.lock();
        audit.update_from_autonomy();
        let safety_score = audit.safety_score();

        unsafe {
            crate::uart_print(b"Autonomous Operation:\n");
            crate::uart_print(b"  Total decisions: ");
            self.print_number_simple(audit.total_decisions as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Autonomous: ");
            self.print_number_simple(audit.autonomous_decisions as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Manual interventions: ");
            self.print_number_simple(audit.manual_interventions as u64);
            crate::uart_print(b"\n\n");

            crate::uart_print(b"Safety Metrics:\n");
            crate::uart_print(b"  Watchdog triggers: ");
            self.print_number_simple(audit.watchdog_triggers as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Rate limit hits: ");
            self.print_number_simple(audit.rate_limit_hits as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Hard limit violations: ");
            self.print_number_simple(audit.hard_limit_violations as u64);
            crate::uart_print(b" (ZERO TOLERANCE)\n");

            crate::uart_print(b"  Rollbacks: ");
            self.print_number_simple(audit.rollbacks_performed as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Safety score: ");
            self.print_number_simple(safety_score as u64);
            crate::uart_print(b"/100\n\n");

            crate::uart_print(b"Performance Metrics:\n");
            crate::uart_print(b"  Avg reward: ");
            if audit.avg_reward < 0 {
                crate::uart_print(b"-");
                self.print_number_simple((-audit.avg_reward) as u64);
            } else {
                self.print_number_simple(audit.avg_reward as u64);
            }
            crate::uart_print(b"\n");

            crate::uart_print(b"  Prediction accuracy: ");
            self.print_number_simple(audit.prediction_accuracy as u64);
            crate::uart_print(b"%\n");

            crate::uart_print(b"  Learning updates: ");
            self.print_number_simple(audit.learning_updates as u64);
            crate::uart_print(b"\n\n");

            crate::uart_print(b"Incident Summary:\n");
            crate::uart_print(b"  Critical: ");
            self.print_number_simple(audit.critical_incidents as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Errors: ");
            self.print_number_simple(audit.error_incidents as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Warnings: ");
            self.print_number_simple(audit.warning_incidents as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"\n========================================\n");
            crate::uart_print(b"Package ready for third-party review\n");
            crate::uart_print(b"========================================\n\n");
        }

        drop(audit);
    }

    /// Generate transparency report
    pub(crate) fn compliance_transparency(&self, _args: &[&str]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"TRANSPARENCY REPORT\n");
            crate::uart_print(b"========================================\n\n");
            crate::uart_print(b"Period: Last 24 hours\n\n");
        }

        let start_ts = crate::time::get_timestamp_us().saturating_sub(86400_000_000);
        let end_ts = crate::time::get_timestamp_us();
        let report = crate::compliance::TransparencyReport::generate(start_ts, end_ts);

        unsafe {
            crate::uart_print(b"Usage Statistics:\n");
            crate::uart_print(b"  Uptime: ");
            self.print_number_simple(report.uptime_seconds);
            crate::uart_print(b" seconds\n");

            crate::uart_print(b"  Autonomous operation: ");
            self.print_number_simple(report.autonomous_percentage as u64);
            crate::uart_print(b"%\n");

            crate::uart_print(b"  Total operations: ");
            self.print_number_simple(report.total_operations as u64);
            crate::uart_print(b"\n\n");

            crate::uart_print(b"Safety Statistics:\n");
            crate::uart_print(b"  Safety score: ");
            self.print_number_simple(report.safety_score as u64);
            crate::uart_print(b"/100\n");

            crate::uart_print(b"  Zero-tolerance violations: ");
            self.print_number_simple(report.zero_tolerance_violations as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Incidents resolved: ");
            self.print_number_simple(report.incidents_resolved as u64);
            crate::uart_print(b"\n\n");

            crate::uart_print(b"Performance Statistics:\n");
            crate::uart_print(b"  Avg accuracy: ");
            self.print_number_simple(report.avg_accuracy as u64);
            crate::uart_print(b"%\n");

            crate::uart_print(b"  Performance vs baseline: ");
            if report.performance_improvement >= 0 {
                crate::uart_print(b"+");
                self.print_number_simple(report.performance_improvement as u64);
            } else {
                crate::uart_print(b"-");
                self.print_number_simple((-report.performance_improvement) as u64);
            }
            crate::uart_print(b"%\n\n");

            crate::uart_print(b"Model Updates:\n");
            crate::uart_print(b"  Versions deployed: ");
            self.print_number_simple(report.model_versions_deployed as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Rollbacks: ");
            self.print_number_simple(report.rollbacks_due_to_issues as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"\n========================================\n");
            crate::uart_print(b"Report generated for stakeholder review\n");
            crate::uart_print(b"========================================\n\n");
        }
    }

    /// Show pre-deployment safety checklist
    pub(crate) fn compliance_checklist(&self, _args: &[&str]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"PRE-DEPLOYMENT SAFETY CHECKLIST\n");
            crate::uart_print(b"========================================\n\n");
        }

        let checklist = crate::compliance::SAFETY_CHECKLIST.lock();
        let completion = checklist.completion_percentage();
        let prod_ready = checklist.is_production_ready();

        unsafe {
            crate::uart_print(b"Core Safety (CRITICAL):\n");
            crate::uart_print(b"  [");
            if checklist.hard_limits_tested { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Hard limits tested\n");

            crate::uart_print(b"  [");
            if checklist.watchdog_functional { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Watchdog functional\n");

            crate::uart_print(b"  [");
            if checklist.rate_limiters_verified { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Rate limiters verified\n");

            crate::uart_print(b"  [");
            if checklist.audit_log_integrity { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Audit log integrity\n");

            crate::uart_print(b"  [");
            if checklist.human_override_tested { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Human override tested\n\n");

            crate::uart_print(b"Learning Safety:\n");
            crate::uart_print(b"  [");
            if checklist.ood_detection_functional { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] OOD detection functional\n");

            crate::uart_print(b"  [");
            if checklist.adversarial_testing_passed { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Adversarial testing passed\n");

            crate::uart_print(b"  [");
            if checklist.reward_tampering_detection { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Reward tampering detection\n\n");

            crate::uart_print(b"Operational Safety:\n");
            crate::uart_print(b"  [");
            if checklist.incremental_autonomy_phases { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Incremental autonomy phases\n");

            crate::uart_print(b"  [");
            if checklist.circuit_breakers_tested { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Circuit breakers tested\n");

            crate::uart_print(b"  [");
            if checklist.rollback_capability { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Rollback capability\n\n");

            crate::uart_print(b"Monitoring:\n");
            crate::uart_print(b"  [");
            if checklist.anomaly_detection_enabled { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Anomaly detection enabled\n");

            crate::uart_print(b"  [");
            if checklist.alerting_system_configured { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Alerting system configured\n\n");

            crate::uart_print(b"Documentation:\n");
            crate::uart_print(b"  [");
            if checklist.compliance_verified { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Compliance verified\n");

            crate::uart_print(b"  [");
            if checklist.incident_runbook_reviewed { crate::uart_print(b"[OK]"); } else { crate::uart_print(b" "); }
            crate::uart_print(b"] Incident runbook reviewed\n\n");

            crate::uart_print(b"========================================\n");
            crate::uart_print(b"Completion: ");
            self.print_number_simple(completion as u64);
            crate::uart_print(b"%\n");

            crate::uart_print(b"Production Ready: ");
            if prod_ready {
                crate::uart_print(b"[OK] YES (all critical items passed)\n");
            } else {
                crate::uart_print(b"[X] NO (critical items missing)\n");
            }
            crate::uart_print(b"========================================\n\n");
        }

        drop(checklist);
    }

    /// Show incident log
    pub(crate) fn compliance_incidents(&self, args: &[&str]) {
        unsafe {
            crate::uart_print(b"\n");
            crate::uart_print(b"========================================\n");
            crate::uart_print(b"INCIDENT LOG\n");
            crate::uart_print(b"========================================\n\n");
        }

        let incident_log = crate::compliance::INCIDENT_LOG.lock();

        let severity_filter = if !args.is_empty() {
            match args[0] {
                "critical" => Some(crate::compliance::IncidentSeverity::Critical),
                "error" => Some(crate::compliance::IncidentSeverity::Error),
                "warning" => Some(crate::compliance::IncidentSeverity::Warning),
                _ => None,
            }
        } else {
            None
        };

        let incidents = incident_log.get_incidents(severity_filter);

        unsafe {
            crate::uart_print(b"Total Incidents: ");
            self.print_number_simple(incidents.len() as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Critical: ");
            self.print_number_simple(incident_log.count_by_severity(crate::compliance::IncidentSeverity::Critical) as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Errors: ");
            self.print_number_simple(incident_log.count_by_severity(crate::compliance::IncidentSeverity::Error) as u64);
            crate::uart_print(b"\n");

            crate::uart_print(b"  Warnings: ");
            self.print_number_simple(incident_log.count_by_severity(crate::compliance::IncidentSeverity::Warning) as u64);
            crate::uart_print(b"\n\n");

            if incidents.is_empty() {
                crate::uart_print(b"No incidents recorded.\n");
            } else {
                crate::uart_print(b"Recent Incidents:\n");
                for (i, incident) in incidents.iter().take(10).enumerate() {
                    crate::uart_print(b"  ");
                    self.print_number_simple((i + 1) as u64);
                    crate::uart_print(b". [");

                    match incident.severity {
                        crate::compliance::IncidentSeverity::Critical => crate::uart_print(b"CRITICAL"),
                        crate::compliance::IncidentSeverity::Error => crate::uart_print(b"ERROR   "),
                        crate::compliance::IncidentSeverity::Warning => crate::uart_print(b"WARNING "),
                    }

                    crate::uart_print(b"] Type=");
                    self.print_number_simple(incident.incident_type as u64);
                    crate::uart_print(b" ");

                    if incident.resolved {
                        crate::uart_print(b"(Resolved)");
                    } else {
                        crate::uart_print(b"(Open)");
                    }
                    crate::uart_print(b"\n");
                }
            }

            crate::uart_print(b"\n========================================\n\n");
        }

        drop(incident_log);
    }
}
