// Helpers for actorctl commands (actor-critic policy control)

impl super::Shell {
    pub(crate) fn actorctl_cmd(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: actorctl <status|policy|sample|lambda N|natural on/off|kl N|on|off>\n"); } return; }
        match args[0] {
            "status" => { crate::meta_agent::print_actor_critic_status(); }
            "policy" => {
                let params = crate::meta_agent::get_policy_params();
                unsafe {
                    crate::uart_print(b"\n=== Current Policy Parameters ===\n\n");
                    crate::uart_print(b"Gaussian Policy (means +/- stddevs):\n");
                    crate::uart_print(b"  Memory: mean=");
                }
                if params.memory_mean < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-params.memory_mean) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(params.memory_mean as u64); }
                unsafe { crate::uart_print(b" stddev="); } self.print_number_simple(params.memory_stddev as u64);
                unsafe { crate::uart_print(b"\n  Scheduling: mean="); }
                if params.scheduling_mean < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-params.scheduling_mean) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(params.scheduling_mean as u64); }
                unsafe { crate::uart_print(b" stddev="); } self.print_number_simple(params.scheduling_stddev as u64);
                unsafe { crate::uart_print(b"\n  Command: mean="); }
                if params.command_mean < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-params.command_mean) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(params.command_mean as u64); }
                unsafe { crate::uart_print(b" stddev="); } self.print_number_simple(params.command_stddev as u64);
                unsafe { crate::uart_print(b"\n\n"); }
            }
            "sample" => {
                let state = crate::meta_agent::collect_telemetry();
                let action = crate::meta_agent::actor_sample_action(&state);
                unsafe { crate::uart_print(b"\n[ACTOR] Sampled action from policy:\n"); crate::uart_print(b"  Memory: "); }
                if action.memory_directive < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-action.memory_directive) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(action.memory_directive as u64); }
                unsafe { crate::uart_print(b"\n  Scheduling: "); }
                if action.scheduling_directive < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-action.scheduling_directive) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(action.scheduling_directive as u64); }
                unsafe { crate::uart_print(b"\n  Command: "); }
                if action.command_directive < 0 { unsafe { crate::uart_print(b"-"); } self.print_number_simple((-action.command_directive) as u64); }
                else { unsafe { crate::uart_print(b"+"); } self.print_number_simple(action.command_directive as u64); }
                unsafe { crate::uart_print(b"\n  Log Prob: "); } self.print_number_simple(action.log_prob.abs() as u64); unsafe { crate::uart_print(b"\n\n"); }
            }
            "lambda" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: actorctl lambda <value 0-1000>\n"); } return; }
                let lambda_milli = args[1].parse::<u16>().unwrap_or(800).min(1000);
                let lambda_q88 = ((lambda_milli as i32 * 256) / 1000) as i16;
                let mut config = crate::meta_agent::get_actor_critic_config();
                config.lambda = lambda_q88; crate::meta_agent::set_actor_critic_config(config);
                unsafe { crate::uart_print(b"[ACTOR] Lambda set to "); } self.print_number_simple(lambda_milli as u64); unsafe { crate::uart_print(b"/1000\n"); }
            }
            "natural" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: actorctl natural <on|off>\n"); } return; }
                let mut config = crate::meta_agent::get_actor_critic_config();
                config.natural_gradient = args[1] == "on"; crate::meta_agent::set_actor_critic_config(config);
                unsafe { crate::uart_print(b"[ACTOR] Natural gradient: "); crate::uart_print(if config.natural_gradient { b"ON\n" } else { b"OFF\n" }); }
            }
            "kl" => {
                if args.len() < 2 { unsafe { crate::uart_print(b"Usage: actorctl kl <threshold 0-100>\n"); } return; }
                let kl_milli = args[1].parse::<u16>().unwrap_or(10).min(100);
                let kl_q88 = ((kl_milli as i32 * 256) / 1000) as i16;
                let mut config = crate::meta_agent::get_actor_critic_config();
                config.kl_threshold = kl_q88; crate::meta_agent::set_actor_critic_config(config);
                unsafe { crate::uart_print(b"[ACTOR] KL threshold set to "); } self.print_number_simple(kl_milli as u64); unsafe { crate::uart_print(b"/1000\n"); }
            }
            "on" => {
                let mut config = crate::meta_agent::get_actor_critic_config();
                config.enabled = true; crate::meta_agent::set_actor_critic_config(config);
                unsafe { crate::uart_print(b"[ACTOR] Actor-critic ENABLED\n"); }
            }
            "off" => {
                let mut config = crate::meta_agent::get_actor_critic_config();
                config.enabled = false; crate::meta_agent::set_actor_critic_config(config);
                unsafe { crate::uart_print(b"[ACTOR] Actor-critic DISABLED\n"); }
            }
            _ => unsafe { crate::uart_print(b"Usage: actorctl <status|policy|sample|lambda N|natural on/off|kl N|on|off>\n"); }
        }
    }
}

