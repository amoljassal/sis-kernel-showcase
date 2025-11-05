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
  type: 'metric' | 'marker' | 'banner' | 'shell';
  [key: string]: any;
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

export const healthApi = {
  async check(): Promise<{ status: string; version: string }> {
    const response = await api.get('/health');
    return response.data;
  },
};
