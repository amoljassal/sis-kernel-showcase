# E2E Test Issues - Executive Summary

**Date:** November 6, 2025
**Status:** E2E Tests BLOCKED - Architectural Mismatch

## Problem Statement

All 63 Playwright E2E tests fail with timeout errors. Tests cannot find UI elements like "Start Replay" button.

## Root Cause Analysis

### Layer 1: Playwright Configuration Issue
**Problem:** Playwright config starts only Vite dev server, not Tauri backend
**Why it matters:** Desktop app calls `invoke('check_daemon')` which requires Tauri Rust backend
**Fix Applied:** Changed `playwright.config.ts` line 24 from `pnpm dev` to `pnpm tauri:dev`

### Layer 2: Missing Cargo Features
**Problem:** Tauri Cargo.toml missing window management features
**Error:** Build failed with "features do not match allowlist" error
**Fix Applied:** Added 8 window features to `src-tauri/Cargo.toml`:
- window-close
- window-hide
- window-maximize
- window-minimize
- window-show
- window-start-dragging
- window-unmaximize
- window-unminimize

### Layer 3: Tauri Compilation Failure
**Problem:** Tauri backend build process killed (SIGKILL)
**Likely Cause:** Memory/resource constraints during Rust compilation
**Impact:** Cannot test with Tauri backend even after config fixes

### Layer 4: Architectural Mismatch (CRITICAL)
**Problem:** Playwright designed for browser testing, but app uses Tauri (native desktop)
**Why it matters:**
- Tauri apps run in OS-native windows with embedded webview
- Playwright tests Chrome/Firefox/WebKit browsers
- Current E2E tests assume pure web app architecture
- Tauri requires WebDriver or custom test harness

## Files Modified

1. **apps/desktop/playwright.config.ts**
   - Line 24: `command: 'pnpm dev'` → `command: 'pnpm tauri:dev'`

2. **apps/desktop/src-tauri/Cargo.toml**
   - Line 14: Added 8 window management features to tauri dependency

## Testing Implications

### Current State
- ✅ Daemon Backend: Fully functional, all APIs working
- ✅ React Frontend Code: Compiles and loads successfully
- ❌ Tauri Backend: Won't compile (resource constraints)
- ❌ E2E Test Suite: Incompatible with Tauri architecture

### Why Tests Fail (Screenshot Evidence)
Tests show app stuck on "Daemon Not Running" screen:
- Header: "Daemon: Disconnected" (red badge)
- Main content: "Daemon Not Running" error message
- Only "Launch Daemon" button visible
- Missing: All tabs, metrics, graphs, "Start Replay" button

This happens because:
1. Tauri backend doesn't run (compilation fails)
2. Frontend's `invoke('check_daemon')` call fails
3. App shows error screen instead of main interface
4. Tests timeout waiting for elements that never render

## Recommended Solutions

### Option A: Use Tauri's Native Testing Tools (Recommended for Desktop App)
**Approach:** Replace Playwright with Tauri-compatible testing
- Use Tauri's WebDriver integration
- Or use Tauri's test mode with automation libraries
- Requires rewriting all 63 E2E tests

**Pros:**
- Tests actual desktop app behavior
- More authentic user experience
- Proper Tauri backend testing

**Cons:**
- High effort (rewrite all tests)
- Different test tooling/infrastructure
- Tauri testing less mature than Playwright

### Option B: Convert to Pure Web App (Recommended for E2E Testing)
**Approach:** Remove Tauri dependency, make it browser-based
- Frontend calls daemon APIs directly via HTTP
- Remove `invoke('check_daemon')` Tauri calls
- Use direct `fetch()` or axios to daemon
- Keep existing Playwright tests

**Changes Required:**
- Modify `src/App.tsx` lines 52-63: Replace `invoke()` with HTTP calls
- Remove Tauri backend entirely (`src-tauri/` directory)
- Update package.json scripts
- Existing E2E tests work as-is

**Pros:**
- Works with existing Playwright tests
- Simpler architecture
- Better for web-based deployment
- No compilation issues

**Cons:**
- Loses desktop app benefits (native integration, packaging)
- No auto-launch daemon feature
- Requires browser instead of standalone app

### Option C: Manual Testing Only
**Approach:** Accept E2E tests won't work, rely on manual QA
- Test desktop app manually with `pnpm tauri:dev`
- Document manual test procedures
- Use Playwright for daemon API tests only (not UI)

**Pros:**
- Fastest to implement
- Works around Tauri compilation issues

**Cons:**
- No automated UI regression detection
- Manual testing time-consuming
- Harder to maintain quality

## Immediate Next Steps

1. **Decision:** Choose Option A, B, or C based on product requirements
   - Is desktop app packaging critical? → Option A
   - Is automated E2E testing critical? → Option B
   - Limited resources/time? → Option C

2. **If Option B chosen (Web App):**
   ```bash
   # Remove Tauri
   rm -rf apps/desktop/src-tauri

   # Update App.tsx to use HTTP instead of Tauri invoke
   # Replace invoke('check_daemon') with fetch('http://localhost:8871/health')

   # Test with existing E2E suite
   pnpm test:e2e
   ```

3. **If Option A chosen (Keep Tauri):**
   - Research Tauri WebDriver setup
   - Rewrite E2E tests using Tauri test harness
   - Fix Tauri compilation issues (more memory, build flags)

4. **If Option C chosen (Manual Only):**
   - Document manual test procedures
   - Create test checklist for QA
   - Run manual smoke tests before releases

## Impact Assessment

**Production Readiness:**
- Daemon: ✅ Ready (all 50 compilation errors fixed, APIs working)
- Frontend: ✅ Likely Ready (code compiles, UI renders)
- Desktop Packaging: ❌ Blocked (Tauri won't compile)
- Automated Testing: ❌ Blocked (architectural mismatch)

**Recommendation:** Convert to pure web app (Option B) for fastest path to working E2E tests and deployment. Desktop packaging can be added later if needed.

## References

- Main Report: `docs/INTEGRATION_TEST_REPORT.md`
- Playwright Config: `apps/desktop/playwright.config.ts`
- Tauri Backend: `apps/desktop/src-tauri/src/main.rs`
- Frontend Entry: `apps/desktop/src/App.tsx:52-63`
