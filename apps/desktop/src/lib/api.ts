/**
 * API client for communicating with sisctl daemon
 */

import axios from 'axios';
import type { components } from 'protos/src/schema';

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

export interface MetricBatchEvent {
  type: 'metric_batch';
  points: BatchedMetricPoint[];
  dropped_count?: number;
  seq?: number;
}

export interface BatchedMetricPoint {
  name: string;
  ts: number; // Unix timestamp in milliseconds
  value: number;
}

export type QemuEvent =
  | { type: 'state_changed'; state: QemuState; timestamp: number }
  | { type: 'parsed'; event: ParsedEvent }
  | { type: 'raw_line'; line: string; timestamp: number }
  | SelfCheckStartedEvent
  | SelfCheckTestEvent
  | SelfCheckCompletedEvent
  | SelfCheckCanceledEvent
  | QemuExitedEvent
  | MetricBatchEvent;

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

// Metrics types from generated schema
export type MetricPoint = components['schemas']['MetricPoint'];
export type SeriesStats = components['schemas']['SeriesStats'];
export type SeriesMetadata = components['schemas']['SeriesMetadata'];
export type QueryResult = components['schemas']['QueryResult'];

export interface MetricsStreamsQuery {
  prefix?: string;
}

export interface MetricsQueryParams {
  name: string;
  from?: number; // Unix timestamp in milliseconds
  to?: number; // Unix timestamp in milliseconds
  maxPoints?: number; // 100-5000, default 1000
}

export const metricsApi = {
  async listStreams(query?: MetricsStreamsQuery): Promise<SeriesMetadata[]> {
    const response = await api.get<SeriesMetadata[]>('/api/v1/metrics/streams', {
      params: query,
    });
    return response.data;
  },

  async query(params: MetricsQueryParams): Promise<QueryResult> {
    const response = await api.get<QueryResult>('/api/v1/metrics/query', {
      params: {
        name: params.name,
        from: params.from,
        to: params.to,
        maxPoints: params.maxPoints,
      },
    });
    return response.data;
  },
};

// Autonomy types
export interface AutonomyStatus {
  enabled: boolean;
  mode: string; // "active", "safe_mode", "learning_frozen"
  interval_ms: number;
  conf_threshold: number;
  total_decisions: number;
  accepted: number;
  deferred: number;
  watchdog_resets: number;
}

export interface AutonomyDecision {
  id: string;
  timestamp: number; // Unix timestamp in ms
  action: string;
  confidence: number;
  reward?: number;
  executed: boolean;
  reason?: string;
}

export interface AttentionWeight {
  feature: string;
  weight: number;
  value: string;
}

export interface ExplainResponse {
  id: string;
  action: string;
  confidence: number;
  attention: AttentionWeight[];
  reasoning: string;
}

export interface PreviewResponse {
  directives: string[];
  confidence: number;
  would_execute: boolean;
  warnings: string[];
}

export interface WhatIfResponse {
  baseline: PreviewResponse;
  scenario: PreviewResponse;
  diff: string[];
}

// Autonomy API
export const autonomyApi = {
  async turnOn(): Promise<AutonomyStatus> {
    const response = await api.post<AutonomyStatus>('/api/v1/autonomy/on');
    return response.data;
  },

  async turnOff(): Promise<AutonomyStatus> {
    const response = await api.post<AutonomyStatus>('/api/v1/autonomy/off');
    return response.data;
  },

  async reset(): Promise<AutonomyStatus> {
    const response = await api.post<AutonomyStatus>('/api/v1/autonomy/reset');
    return response.data;
  },

  async setInterval(interval_ms: number): Promise<AutonomyStatus> {
    const response = await api.post<AutonomyStatus>('/api/v1/autonomy/interval', {
      interval_ms,
    });
    return response.data;
  },

  async setThreshold(threshold: number): Promise<AutonomyStatus> {
    const response = await api.post<AutonomyStatus>('/api/v1/autonomy/conf-threshold', {
      threshold,
    });
    return response.data;
  },

  async status(): Promise<AutonomyStatus> {
    const response = await api.get<AutonomyStatus>('/api/v1/autonomy/status');
    return response.data;
  },

  async audit(last: number = 100): Promise<AutonomyDecision[]> {
    const response = await api.get<AutonomyDecision[]>('/api/v1/autonomy/audit', {
      params: { last },
    });
    return response.data;
  },

  async explain(id: string): Promise<ExplainResponse> {
    const response = await api.get<ExplainResponse>('/api/v1/autonomy/explain', {
      params: { id },
    });
    return response.data;
  },

  async preview(count: number = 10): Promise<PreviewResponse> {
    const response = await api.post<PreviewResponse>('/api/v1/autonomy/preview', {
      count,
    });
    return response.data;
  },

  async whatIf(overrides: Record<string, any>): Promise<WhatIfResponse> {
    const response = await api.post<WhatIfResponse>('/api/v1/autonomy/whatif', {
      overrides,
    });
    return response.data;
  },
};

// Memory approval types
export interface MemoryApprovalStatus {
  enabled: boolean;
  query_mode: boolean; // dry-run mode
  pending_count: number;
  total_approved: number;
  total_rejected: number;
}

export interface PendingOperation {
  id: string;
  op_type: string; // "alloc", "free", "remap"
  confidence: number;
  risk: string; // "low", "medium", "high"
  reason: string;
  timestamp: number; // Unix timestamp in ms
  size_bytes?: number;
  address?: string;
}

// Memory API
export const memoryApi = {
  async getApprovals(limit: number = 100): Promise<PendingOperation[]> {
    const response = await api.get<PendingOperation[]>('/api/v1/mem/approvals', {
      params: { limit },
    });
    return response.data;
  },

  async toggleApproval(action: 'on' | 'off' | 'status'): Promise<MemoryApprovalStatus> {
    const response = await api.post<MemoryApprovalStatus>('/api/v1/mem/approval', {
      action,
    });
    return response.data;
  },

  async approve(n: number = 1): Promise<MemoryApprovalStatus> {
    const response = await api.post<MemoryApprovalStatus>('/api/v1/mem/approve', {
      n,
    });
    return response.data;
  },

  async reject(id?: string): Promise<MemoryApprovalStatus> {
    const response = await api.post<MemoryApprovalStatus>('/api/v1/mem/reject', {
      id,
    });
    return response.data;
  },
};
