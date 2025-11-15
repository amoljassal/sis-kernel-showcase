AI‑Native Kernel (QEMU Demo) — One‑Page Summary

Summary
- I build working demos fast using AI as my build partner. This kernel demo boots with one command and produces “image → top‑5 labels” with timings in seconds. It’s designed to showcase rapid prototyping, not to be a production OS.

What It Does
- Runs a tiny pipeline inside a kernel demo and prints results instantly.
- Shows step timings (normalize/model/total) for clear performance insight.
- Includes a live “safety key” rotation to demonstrate basic control security.

Why It Matters
- Proves speed: idea → running system quickly, with clear outcomes.
- Proves clarity: built‑in timing and simple commands anyone can run.
- Proves hand‑off: a clean repo and a minimal interface senior engineers can extend.

How To Run (copy‑paste)
- Build + boot (QEMU): `BRINGUP=1 ./scripts/uefi_run.sh`
- In the SIS shell: `imagedemo` → prints top‑5 labels + timing metrics.
- Safety moment: `ctlkey` → `ctlkey 0x53535F4354524C22` → `ctlkey` (confirm key changed).

What You’ll See
- Five labels with scores (e.g., person, tree, car…).
- Three timing metrics: normalize_us, model_us, total_us.
- Safety key shown before and after rotation.

Optional: Deterministic Mode
- Build with feature: `SIS_FEATURES="deterministic" BRINGUP=1 ./scripts/uefi_run.sh`.
- Commands: `graphctl create` → `graphctl add-channel 64` → `det on 50000 200000 200000` → `det status` → `graphctl start 10` → `det status`.
- Purpose: shows “train‑timetable” behavior (admission + miss counter) in plain terms.

Hand‑Off Plan (Week 1–2 if a team adopts it)
- Add a tiny Python adapter so normal scripts can call the kernel pipeline like a function.
- Add two common operators (resize/normalize, top‑K) and a simple HTML/text summary report.
- Keep the repo clean and the demo script unchanged (one command to run).

Links & Contact
- Repo (showcase): https://github.com/amoljassal/sis-kernel-showcase
- LinkedIn: https://www.linkedin.com/in/amoljassal/
- Email: (add your preferred email)

Export to PDF
- Open this file in any Markdown preview (e.g., VS Code), then “Print” → “Save as PDF”.
- Or open the HTML version in docs/one-pager and “File” → “Print” → “Save as PDF”.

