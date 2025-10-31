// Helpers for LLM control commands (feature: llm)

#[cfg(feature = "llm")]
impl super::Shell {
    pub(crate) fn llmctl_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: llmctl load [--wcet-cycles N] [--model ID] [--hash 0xHEX..64] [--sig 0xHEX..128] | budget [--wcet-cycles N] [--period-ns N] [--max-tokens-per-period N]\n"); }
            return;
        }
        match args[0] {
            "load" => { self.llmctl_load_cmd(args); }
            "budget" => {
                let mut wcet: Option<u64> = None;
                let mut period: Option<u64> = None;
                let mut max_per: Option<usize> = None;
                let mut i = 1;
                while i < args.len() {
                    match args[i] {
                        "--wcet-cycles" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u64>(){ wcet=Some(v);} } },
                        "--period-ns" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u64>(){ period=Some(v);} } },
                        "--max-tokens-per-period" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<usize>(){ max_per=Some(v);} } },
                        _ => {}
                    }
                    i+=1;
                }
                crate::llm::configure_budget(wcet, period, max_per);
                unsafe { crate::uart_print(b"[LLM] budget configured\n"); }
            }
            "status" => {
                #[cfg(feature = "deterministic")]
                {
                    let (used, acc, rej, misses, p99) = crate::deterministic::llm_get_status();
                    unsafe {
                        crate::uart_print(b"[LLM][DET] used_ppm="); self.print_number_simple(used as u64);
                        crate::uart_print(b" accepted="); self.print_number_simple(acc as u64);
                        crate::uart_print(b" rejected="); self.print_number_simple(rej as u64);
                        crate::uart_print(b" deadline_misses="); self.print_number_simple(misses as u64);
                        crate::uart_print(b" jitter_p99_ns="); self.print_number_simple(p99 as u64);
                        crate::uart_print(b"\n");
                    }
                }
                #[cfg(not(feature = "deterministic"))]
                unsafe { crate::uart_print(b"[LLM] deterministic feature not enabled\n"); }
            }
            "audit" => { crate::llm::audit_print(); }
            _ => unsafe { crate::uart_print(b"Usage: llmctl load [--wcet-cycles N] [--model ID] [--hash 0xHEX..64] [--sig 0xHEX..128] | budget [--wcet-cycles N] [--period-ns N] [--max-tokens-per-period N]\n"); },
        }
    }

    pub(crate) fn llmctl_load_cmd(&self, args: &[&str]) {
        let mut wcet: Option<u64> = None;
        let mut model_id: Option<u32> = None;
        let mut hash_bytes: Option<[u8;32]> = None;
        let mut sig_bytes: Option<[u8;64]> = None;
        let mut ctx: Option<u32> = None;
        let mut vocab: Option<u32> = None;
        let mut quant: Option<crate::llm::Quantization> = None;
        let mut name: Option<alloc::string::String> = None;
        let mut rev: Option<u32> = None;
        let mut size_bytes: Option<usize> = None;
        let mut i = 1;
        while i < args.len() {
            match args[i] {
                "--wcet-cycles" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u64>(){ wcet=Some(v);} } },
                "--model" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u32>(){ model_id=Some(v);} } },
                "--hash" => { i+=1; if i<args.len(){ if let Some(b)=Self::parse_hex_fixed::<32>(args[i]){ hash_bytes=Some(b);} } },
                "--sig" => { i+=1; if i<args.len(){ if let Some(b)=Self::parse_hex_fixed::<64>(args[i]){ sig_bytes=Some(b);} } },
                "--ctx" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u32>(){ ctx=Some(v);} } },
                "--vocab" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u32>(){ vocab=Some(v);} } },
                "--quant" => { i+=1; if i<args.len(){ quant=match args[i].to_ascii_lowercase().as_str(){"q4_0"=>Some(crate::llm::Quantization::Q4_0),"q4_1"=>Some(crate::llm::Quantization::Q4_1),"int8"=>Some(crate::llm::Quantization::Int8),"fp16"=>Some(crate::llm::Quantization::FP16),"fp32"=>Some(crate::llm::Quantization::FP32),_=>None}; } },
                "--name" => { i+=1; if i<args.len(){ let mut s=alloc::string::String::new(); s.push_str(args[i]); name=Some(s);} },
                "--rev" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<u32>(){ rev=Some(v);} } },
                "--size-bytes" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<usize>(){ size_bytes=Some(v);} } },
                _ => {}
            }
            i+=1;
        }
        let ok = if model_id.is_some() && hash_bytes.is_some() {
            let mid = model_id.unwrap();
            let hb = hash_bytes.unwrap();
            let sb = sig_bytes.unwrap_or([0u8;64]);
            let sz = size_bytes.unwrap_or(1024);
            crate::llm::load_model_package(mid, hb, sb, sz)
        } else if ctx.is_some() || vocab.is_some() || quant.is_some() || name.is_some() || rev.is_some() || size_bytes.is_some() {
            let mid = model_id.unwrap_or(0);
            let meta = crate::llm::ModelMeta { id: mid, name, ctx_len: ctx.unwrap_or(2048), vocab_size: vocab.unwrap_or(50_000), quant: quant.unwrap_or(crate::llm::Quantization::Int8), revision: rev, size_bytes: size_bytes.unwrap_or(1_048_576) };
            crate::llm::load_model_with_meta(Some(meta), wcet, None)
        } else if model_id.is_some() {
            crate::llm::load_model_meta(model_id, wcet, None)
        } else {
            crate::llm::load_model(wcet)
        };
        unsafe { if ok { crate::uart_print(b"[LLM] model loaded\n"); } else { crate::uart_print(b"[LLM] model load failed\n"); } }
    }

    pub(crate) fn parse_hex_fixed<const N: usize>(s: &str) -> Option<[u8; N]> {
        let hex = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
        if hex.len() != N * 2 { return None; }
        let mut out = [0u8; N];
        let bytes = hex.as_bytes();
        for i in 0..N {
            let hi = bytes[i*2];
            let lo = bytes[i*2+1];
            let hn = match hi { b'0'..=b'9' => hi - b'0', b'a'..=b'f' => hi - b'a' + 10, b'A'..=b'F' => hi - b'A' + 10, _ => 0xFF };
            let ln = match lo { b'0'..=b'9' => lo - b'0', b'a'..=b'f' => lo - b'a' + 10, b'A'..=b'F' => lo - b'A' + 10, _ => 0xFF };
            if hn > 15 || ln > 15 { return None; }
            out[i] = (hn << 4) | ln;
        }
        Some(out)
    }

    pub(crate) fn llminfer_cmd(&self, args: &[&str]) {
        if args.is_empty() {
            unsafe { crate::uart_print(b"Usage: llminfer <prompt text> [--max-tokens N]\n"); }
            return;
        }
        let mut max_tokens: Option<usize> = None;
        let mut prompt_parts: heapless::Vec<&str, 32> = heapless::Vec::new();
        let mut i = 0usize;
        while i < args.len() {
            if args[i] == "--max-tokens" {
                i += 1;
                if i < args.len() { if let Ok(v) = args[i].parse::<usize>() { max_tokens = Some(v); } }
            } else {
                let _ = prompt_parts.push(args[i]);
            }
            i += 1;
        }
        let mut prompt = alloc::string::String::new();
        for (k, p) in prompt_parts.iter().enumerate() {
            if k > 0 { prompt.push(' '); }
            prompt.push_str(p);
        }
        let res = crate::llm::infer(&prompt, max_tokens);
        unsafe {
            crate::uart_print(b"[LLM] infer id="); self.print_number_simple(res.infer_id as u64);
            crate::uart_print(b" tokens="); self.print_number_simple(res.tokens_emitted as u64);
            crate::uart_print(b" latency_us="); self.print_number_simple(res.latency_us as u64);
            crate::uart_print(b"\n[LLM] output: ");
            crate::uart_print(res.output.as_bytes());
            crate::uart_print(b"\n");
        }
    }

    pub(crate) fn llmstats_cmd(&self) {
        let (qdmax, total_tokens, misses, last_us) = crate::llm::stats();
        unsafe {
            crate::uart_print(b"[LLM] queue_depth_max="); self.print_number_simple(qdmax as u64);
            crate::uart_print(b" total_tokens="); self.print_number_simple(total_tokens as u64);
            crate::uart_print(b" deadline_misses="); self.print_number_simple(misses as u64);
            crate::uart_print(b" last_latency_us="); self.print_number_simple(last_us as u64);
            crate::uart_print(b"\n");
        }
    }

    pub(crate) fn llm_audit_json_cmd(&self) { crate::llm::audit_print_json(); }

    pub(crate) fn llmsig_cmd(&self, args: &[&str]) {
        if args.len() < 1 {
            unsafe { crate::uart_print(b"Usage: llmsig <model_id>\n"); }
            return;
        }
        let id = match args[0].parse::<u32>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[LLM] invalid model id\n"); } return; } };
        let salt_a: u64 = 0xA5A5_A5A5_A5A5_A5A5;
        let salt_b: u64 = 0x5349_534C_4D4F_444C; // b"SISLMODL"
        let sig = salt_a ^ salt_b ^ (id as u64);
        unsafe { crate::uart_print(b"LLM SIG: "); }
        self.print_hex(sig);
        unsafe { crate::uart_print(b"\n"); }
    }

    pub(crate) fn llmpoll_cmd(&self, args: &[&str]) {
        let max = if !args.is_empty() { args[0].parse::<usize>().unwrap_or(4) } else { 4 };
        let (id, n, done, _items) = crate::llm::ctl_poll(max);
        let (plen, model_id) = crate::llm::ctl_peek_meta(id);
        unsafe {
            crate::uart_print(b"[LLM][POLL] id="); self.print_number_simple(id as u64);
            crate::uart_print(b" n="); self.print_number_simple(n as u64);
            crate::uart_print(b" done="); self.print_number_simple(done as u64);
            crate::uart_print(b" plen="); self.print_number_simple(plen as u64);
            crate::uart_print(b" model=");
            match model_id { Some(mid) => self.print_number_simple(mid as u64), None => crate::uart_print(b"none"), }
            crate::uart_print(b"\n");
        }
    }

    pub(crate) fn llmcancel_cmd(&self, args: &[&str]) {
        if let Some(id_str) = args.get(0) {
            if let Ok(id) = id_str.parse::<u32>() {
                crate::llm::ctl_cancel_id(id as usize);
                unsafe { crate::uart_print(b"[LLM] cancel issued for id="); }
                self.print_number_simple(id as u64);
                unsafe { crate::uart_print(b"\n"); }
                return;
            }
        }
        crate::llm::ctl_cancel();
        unsafe { crate::uart_print(b"[LLM] cancel issued\n"); }
    }

    pub(crate) fn llm_summary_cmd(&self) { crate::llm::ctl_print_sessions(); }

    pub(crate) fn llm_verify_cmd(&self) {
        let ok = crate::llm::verify_demo_model();
        unsafe { if ok { crate::uart_print(b"[LLM][MODEL] verify ok\n"); } else { crate::uart_print(b"[LLM][MODEL] verify FAILED\n"); } }
    }

    pub(crate) fn llm_hash_cmd(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: llmhash <model_id> [size_bytes]\n"); } return; }
        let id = match args[0].parse::<u32>() { Ok(v) => v, Err(_) => { unsafe { crate::uart_print(b"[LLM] invalid model id\n"); } return; } };
        let size = if args.len() >= 2 { args[1].parse::<usize>().unwrap_or(1024) } else { 1024 };
        let hash = crate::llm::demo_hash_for(id, size);
        unsafe { crate::uart_print(b"LLM HASH: 0x"); }
        for b in hash { let hi = (b >> 4) & 0xF; let lo = b & 0xF; let table = b"0123456789ABCDEF"; unsafe { crate::uart_print(&[table[hi as usize]]); crate::uart_print(&[table[lo as usize]]); } }
        unsafe { crate::uart_print(b"\n"); }
    }

    pub(crate) fn llm_key_cmd(&self) {
        #[cfg(feature = "crypto-real")]
        {
            match crate::model::get_verifying_key() {
                Some(pk) => {
                    unsafe { crate::uart_print(b"LLM PUBKEY: 0x"); }
                    let table = b"0123456789abcdef";
                    for b in pk { let hi = (b >> 4) & 0xF; let lo = b & 0xF; unsafe { crate::uart_print(&[table[hi as usize]]); crate::uart_print(&[table[lo as usize]]); } }
                    unsafe { crate::uart_print(b"\n"); }
                }
                None => unsafe { crate::uart_print(b"LLM PUBKEY: <unset>\n"); },
            }
        }
        #[cfg(not(feature = "crypto-real"))]
        unsafe { crate::uart_print(b"[LLM] crypto-real feature not enabled\n"); }
    }

    pub(crate) fn llmstream_cmd(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: llmstream <prompt text> [--max-tokens N] [--chunk N]\n"); } return; }
        let mut max_tokens: Option<usize> = None;
        let mut chunk: usize = 2;
        let mut prompt_parts: heapless::Vec<&str, 32> = heapless::Vec::new();
        let mut i = 0usize;
        while i < args.len() {
            match args[i] {
                "--max-tokens" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<usize>(){ max_tokens=Some(v);} } },
                "--chunk" => { i+=1; if i<args.len(){ if let Ok(v)=args[i].parse::<usize>(){ if v>0 { chunk=v; } } } },
                _ => { let _ = prompt_parts.push(args[i]); }
            }
            i+=1;
        }
        let mut prompt = alloc::string::String::new();
        for (k,p) in prompt_parts.iter().enumerate(){ if k>0 { prompt.push(' ');} prompt.push_str(p);}        
        let res = crate::llm::infer_stream(&prompt, max_tokens, chunk);
        unsafe {
            crate::uart_print(b"[LLM][STREAM] infer id="); self.print_number_simple(res.infer_id as u64);
            crate::uart_print(b" tokens="); self.print_number_simple(res.tokens_emitted as u64);
            crate::uart_print(b" latency_us="); self.print_number_simple(res.latency_us as u64);
            crate::uart_print(b"\n");
        }
    }

    pub(crate) fn llmgraph_cmd(&self, args: &[&str]) {
        if args.is_empty() { unsafe { crate::uart_print(b"Usage: llmgraph <prompt text>\n"); } return; }
        let mut prompt = alloc::string::String::new(); for (k,p) in args.iter().enumerate(){ if k>0 { prompt.push(' ');} prompt.push_str(p);}        
        let mut g = crate::graph::GraphApi::create();
        let in_ch = g.add_channel(crate::graph::ChannelSpec{capacity:64});
        let out_ch = g.add_channel(crate::graph::ChannelSpec{capacity:64});
        let _op = g.add_operator(crate::graph::OperatorSpec{
            id: 42,
            func: crate::graph::op_llm_run,
            in_ch: Some(in_ch),
            out_ch: Some(out_ch),
            priority: 10,
            stage: None,
            in_schema: None,
            out_schema: None,
        });
        unsafe {
            use crate::tensor::{TensorHeader, TensorAlloc};
            let text_bytes = prompt.as_bytes();
            let total = core::mem::size_of::<TensorHeader>() + text_bytes.len();
            if let Some(h) = TensorAlloc::alloc_uninit(total, 64) {
                if let Some(hdr) = h.header_mut() {
                    hdr.version = 1; hdr.dtype = 0; hdr.dims = [0;4]; hdr.strides=[0;4];
                    hdr.data_offset = core::mem::size_of::<TensorHeader>() as u64;
                    hdr.schema_id = 1001; // SCHEMA_TEXT
                    hdr.records = 1; hdr.quality=100; hdr._pad=0; hdr.lineage=0;
                }
                let dst = (h.ptr as usize + core::mem::size_of::<TensorHeader>()) as *mut u8;
                core::ptr::copy_nonoverlapping(text_bytes.as_ptr(), dst, text_bytes.len());
                let _ = g.channel(in_ch).try_enqueue(h);
            }
        }
        g.run_steps(4);
        let out = g.channel(out_ch);
        let mut _drained = 0usize;
        loop {
            if let Some(h) = out.try_dequeue() {
                unsafe {
                    let (data_ptr, data_len) = if let Some(hdr)=h.header(){ ((h.ptr as usize + hdr.data_offset as usize) as *const u8, (h.len.saturating_sub(hdr.data_offset as usize))) } else { (h.ptr as *const u8, h.len) };
                    let sl = core::slice::from_raw_parts(data_ptr, data_len);
                    crate::uart_print(b"[LLM][GRAPH-OUT] chunk: ");
                    crate::uart_print(sl);
                    crate::uart_print(b"\n");
                    crate::tensor::TensorAlloc::dealloc(h, 64);
                }
                _drained += 1;
            } else { break; }
        }
        unsafe { crate::uart_print(b"[LLM][GRAPH] done\n"); }
    }
}
