/**
 * Playwright Global Setup
 * - Starts the sisctl daemon on 127.0.0.1:8871
 * - Waits for /health to become ready
 * - Provides teardown to stop the daemon after tests
 */

import { spawn, ChildProcess } from 'child_process';
import http from 'http';
import fs from 'fs';
import path from 'path';

const DAEMON_PORT = process.env.SISCTL_PORT || '8871';
const DAEMON_HOST = process.env.SISCTL_HOST || '127.0.0.1';
const DAEMON_BASE = `http://${DAEMON_HOST}:${DAEMON_PORT}`;

function waitForHealth(timeoutMs = 20000): Promise<void> {
  const started = Date.now();

  return new Promise((resolve, reject) => {
    const tryOnce = () => {
      const req = http.get(`${DAEMON_BASE}/health`, (res) => {
        if (res.statusCode && res.statusCode >= 200 && res.statusCode < 300) {
          resolve();
        } else {
          if (Date.now() - started > timeoutMs) {
            reject(new Error(`Daemon health not ready: status ${res.statusCode}`));
          } else {
            setTimeout(tryOnce, 500);
          }
        }
      });
      req.on('error', () => {
        if (Date.now() - started > timeoutMs) {
          reject(new Error('Daemon health check failed'));
        } else {
          setTimeout(tryOnce, 500);
        }
      });
    };
    tryOnce();
  });
}

function spawnDaemon(): ChildProcess {
  // Allow overriding the command via env
  const overrideCmd = process.env.SISCTL_CMD;
  if (overrideCmd) {
    const [cmd, ...args] = overrideCmd.split(' ');
    return spawn(cmd, args, { stdio: 'inherit', env: { ...process.env, SISCTL_BIND: `${DAEMON_HOST}:${DAEMON_PORT}` } });
  }

  // Try prebuilt binary first
  const relPaths = [
    'apps/daemon/target/release/sisctl',
    'apps/daemon/target/debug/sisctl',
  ];
  for (const p of relPaths) {
    const abs = path.resolve(process.cwd(), p);
    if (fs.existsSync(abs)) {
      return spawn(abs, [], { stdio: 'inherit', env: { ...process.env, SISCTL_BIND: `${DAEMON_HOST}:${DAEMON_PORT}` } });
    }
  }

  // Fallback to cargo run (may compile if needed)
  return spawn(
    'cargo',
    ['run', '--manifest-path', 'apps/daemon/Cargo.toml', '--bin', 'sisctl'],
    { stdio: 'inherit', env: { ...process.env, SISCTL_BIND: `${DAEMON_HOST}:${DAEMON_PORT}` } }
  );
}

let child: ChildProcess | null = null;

async function globalSetup() {
  // If a daemon is already running, skip spawning
  try {
    await waitForHealth(2000);
    console.log('[global-setup] Daemon already running');
    return;
  } catch {
    // not running yet
  }

  console.log('[global-setup] Starting sisctl daemon...');
  child = spawnDaemon();

  // Wait for health
  await waitForHealth(30000);
  console.log('[global-setup] Daemon is healthy at', DAEMON_BASE);

  // Expose teardown
  return async () => {
    if (child && !child.killed) {
      console.log('[global-teardown] Stopping sisctl daemon');
      try {
        child.kill('SIGTERM');
      } catch {}
    }
  };
}

export default globalSetup;
