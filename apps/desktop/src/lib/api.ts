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

export type QemuEvent =
  | { type: 'state_changed'; state: QemuState; timestamp: number }
  | { type: 'parsed'; event: ParsedEvent }
  | { type: 'raw_line'; line: string; timestamp: number }
  | SelfCheckStartedEvent
  | SelfCheckTestEvent
  | SelfCheckCompletedEvent
  | SelfCheckCanceledEvent;

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
