/**
 * API client for communicating with sisctl daemon
 */

import axios from 'axios';

const DAEMON_URL = import.meta.env.VITE_DAEMON_URL || 'http://localhost:8871';

export const api = axios.create({
  baseURL: DAEMON_URL,
  timeout: 10000,
});

// Types matching daemon API
export interface QemuConfig {
  features?: string[];
  env?: Record<string, string>;
  args?: string[];
  working_dir?: string;
}

export enum QemuState {
  Idle = 'idle',
  Starting = 'starting',
  Running = 'running',
  Stopping = 'stopping',
  Failed = 'failed',
}

export interface QemuStatus {
  state: QemuState;
  pid?: number;
  uptime_secs?: number;
  features: string[];
  error?: string;
  lines_processed: number;
  events_emitted: number;
}

export interface BootMarker {
  marker: string;
  timestamp: number;
}

export interface MetricEvent {
  name: string;
  value: number;
  timestamp: number;
}

export interface ParsedEvent {
  type: 'metric' | 'marker' | 'banner' | 'shell' | 'prompt' | 'test_result';
  [key: string]: any;
}

export interface SelfCheckStartedEvent {
  type: 'self_check_started';
  timestamp: number;
}

export interface SelfCheckTestEvent {
  type: 'self_check_test';
  name: string;
  passed: boolean;
  timestamp: number;
}

export interface SelfCheckCompletedEvent {
  type: 'self_check_completed';
  total: number;
  passed: number;
  failed: number;
  success: boolean;
  timestamp: number;
}

export interface SelfCheckCanceledEvent {
  type: 'self_check_canceled';
  timestamp: number;
}

export interface QemuExitedEvent {
  type: 'qemu_exited';
  code: number | null;
  timestamp: number;
}

export type QemuEvent =
  | { type: 'state_changed'; state: QemuState; timestamp: number }
  | { type: 'parsed'; event: ParsedEvent }
  | { type: 'raw_line'; line: string; timestamp: number }
  | SelfCheckStartedEvent
  | SelfCheckTestEvent
  | SelfCheckCompletedEvent
  | SelfCheckCanceledEvent
  | QemuExitedEvent;

export interface ShellCommandRequest {
  command: string;
  timeout_ms?: number;
}

export interface ShellCommandResponse {
  command: string;
  output: string[];
  success: boolean;
  error?: string;
  execution_time_ms: number;
}

export interface TestResultEntry {
  name: string;
  passed: boolean;
  timestamp: number;
}

export interface SelfCheckResponse {
  tests: TestResultEntry[];
  total: number;
  passed: number;
  failed: number;
  success: boolean;
  execution_time_ms: number;
}

// API methods
export const qemuApi = {
  async run(config: QemuConfig): Promise<void> {
    await api.post('/api/v1/qemu/run', config);
  },

  async stop(): Promise<void> {
    await api.post('/api/v1/qemu/stop');
  },

  async status(): Promise<QemuStatus> {
    const response = await api.get<QemuStatus>('/api/v1/qemu/status');
    return response.data;
  },
};

export const shellApi = {
  async exec(request: ShellCommandRequest): Promise<ShellCommandResponse> {
    const response = await api.post<ShellCommandResponse>(
      '/api/v1/shell/exec',
      request
    );
    return response.data;
  },

  async selfcheck(): Promise<SelfCheckResponse> {
    const response = await api.post<SelfCheckResponse>(
      '/api/v1/shell/selfcheck'
    );
    return response.data;
  },

  async cancelSelfcheck(): Promise<void> {
    await api.post('/api/v1/shell/selfcheck/cancel');
  },
};

export const healthApi = {
  async check(): Promise<{ status: string; version: string }> {
    const response = await api.get('/health');
    return response.data;
  },
};

// Replay types
export enum ReplayState {
  Idle = 'idle',
  Running = 'running',
}

export interface ReplayStatus {
  state: ReplayState;
  source?: string;
  mode?: string;
  progress: number; // 0-100
}

export interface ReplayRequest {
  mode?: string; // 'sample' or 'upload'
  logSource?: string; // Sample name or file identifier
  file?: string; // Base64 encoded file content
  speed?: string; // 'instant', 'fast', 'realtime'
  sample?: string; // DEPRECATED: Use logSource with mode=sample
}

export interface ReplayResponse {
  message: string;
  lines_processed: number;
}

export const replayApi = {
  async start(request: ReplayRequest): Promise<ReplayResponse> {
    const response = await api.post<ReplayResponse>('/api/v1/replay', request);
    return response.data;
  },

  async stop(): Promise<void> {
    await api.post('/api/v1/replay/stop');
  },

  async status(): Promise<ReplayStatus> {
    const response = await api.get<ReplayStatus>('/api/v1/replay/status');
    return response.data;
  },
};
