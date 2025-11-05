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

// ============================================================================
// M4 APIs
// ============================================================================

// Graph types (from OpenAPI schema)
export type GraphState = components['schemas']['GraphState'];
export type CreateGraphResponse = components['schemas']['CreateGraphResponse'];
export type AddChannelRequest = components['schemas']['AddChannelRequest'];
export type AddChannelResponse = components['schemas']['AddChannelResponse'];
export type AddOperatorRequest = components['schemas']['AddOperatorRequest'];
export type AddOperatorResponse = components['schemas']['AddOperatorResponse'];
export type StartGraphRequest = components['schemas']['StartGraphRequest'];
export type StartGraphResponse = components['schemas']['StartGraphResponse'];
export type PredictRequest = components['schemas']['PredictRequest'];
export type PredictResponse = components['schemas']['PredictResponse'];
export type FeedbackRequest = components['schemas']['FeedbackRequest'];
export type FeedbackResponse = components['schemas']['FeedbackResponse'];
export type ExportGraphRequest = components['schemas']['ExportGraphRequest'];
export type ExportGraphResponse = components['schemas']['ExportGraphResponse'];

// Scheduling types
export type Workload = components['schemas']['Workload'];
export type SetPriorityRequest = components['schemas']['SetPriorityRequest'];
export type SetAffinityRequest = components['schemas']['SetAffinityRequest'];
export type SetFeatureRequest = components['schemas']['SetFeatureRequest'];
export type SchedResponse = components['schemas']['SchedResponse'];
export type CircuitBreakerState = components['schemas']['CircuitBreakerState'];

// LLM types
export type LoadModelRequest = components['schemas']['LoadModelRequest'];
export type LoadModelResponse = components['schemas']['LoadModelResponse'];
export type InferRequest = components['schemas']['InferRequest'];
export type InferResponse = components['schemas']['InferResponse'];
export type AuditEntry = components['schemas']['AuditEntry'];
export type LlmStatus = components['schemas']['LlmStatus'];

// Logs types
export type LogEntry = components['schemas']['LogEntry'];
export type RunProfile = components['schemas']['RunProfile'];
export type StartRunRequest = components['schemas']['StartRunRequest'];
export type StartRunResponse = components['schemas']['StartRunResponse'];
export type StopRunResponse = components['schemas']['StopRunResponse'];
export type RunHistoryEntry = components['schemas']['RunHistoryEntry'];

// Graph API
export const graphApi = {
  async create(): Promise<CreateGraphResponse> {
    const response = await api.post<CreateGraphResponse>('/api/v1/graph/create');
    return response.data;
  },

  async addChannel(req: AddChannelRequest): Promise<AddChannelResponse> {
    const response = await api.post<AddChannelResponse>('/api/v1/graph/add-channel', req);
    return response.data;
  },

  async addOperator(req: AddOperatorRequest): Promise<AddOperatorResponse> {
    const response = await api.post<AddOperatorResponse>('/api/v1/graph/add-operator', req);
    return response.data;
  },

  async start(req: StartGraphRequest): Promise<StartGraphResponse> {
    const response = await api.post<StartGraphResponse>('/api/v1/graph/start', req);
    return response.data;
  },

  async predict(req: PredictRequest): Promise<PredictResponse> {
    const response = await api.post<PredictResponse>('/api/v1/graph/predict', req);
    return response.data;
  },

  async feedback(req: FeedbackRequest): Promise<FeedbackResponse> {
    const response = await api.post<FeedbackResponse>('/api/v1/graph/feedback', req);
    return response.data;
  },

  async state(graphId: string): Promise<GraphState> {
    const response = await api.get<GraphState>('/api/v1/graph/state', {
      params: { graphId },
    });
    return response.data;
  },

  async export(req: ExportGraphRequest): Promise<ExportGraphResponse> {
    const response = await api.post<ExportGraphResponse>('/api/v1/graph/export', req);
    return response.data;
  },
};

// Scheduling API
export const schedApi = {
  async workloads(): Promise<Workload[]> {
    const response = await api.get<Workload[]>('/api/v1/sched/workloads');
    return response.data;
  },

  async setPriority(req: SetPriorityRequest): Promise<SchedResponse> {
    const response = await api.post<SchedResponse>('/api/v1/sched/priorities', req);
    return response.data;
  },

  async setAffinity(req: SetAffinityRequest): Promise<SchedResponse> {
    const response = await api.post<SchedResponse>('/api/v1/sched/affinity', req);
    return response.data;
  },

  async setFeature(req: SetFeatureRequest): Promise<SchedResponse> {
    const response = await api.post<SchedResponse>('/api/v1/sched/feature', req);
    return response.data;
  },

  async circuitBreakerStatus(): Promise<CircuitBreakerState> {
    const response = await api.get<CircuitBreakerState>('/api/v1/sched/circuit-breaker');
    return response.data;
  },

  async circuitBreakerReset(): Promise<SchedResponse> {
    const response = await api.post<SchedResponse>('/api/v1/sched/circuit-breaker/reset');
    return response.data;
  },
};

// LLM API
export const llmApi = {
  async load(req: LoadModelRequest): Promise<LoadModelResponse> {
    const response = await api.post<LoadModelResponse>('/api/v1/llm/load', req);
    return response.data;
  },

  async infer(req: InferRequest): Promise<InferResponse> {
    const response = await api.post<InferResponse>('/api/v1/llm/infer', req);
    return response.data;
  },

  async audit(): Promise<AuditEntry[]> {
    const response = await api.get<AuditEntry[]>('/api/v1/llm/audit');
    return response.data;
  },

  async status(): Promise<LlmStatus> {
    const response = await api.get<LlmStatus>('/api/v1/llm/status');
    return response.data;
  },
};

// Logs API
export const logsApi = {
  async tail(params?: { limit?: number; level?: string; source?: string }): Promise<LogEntry[]> {
    const response = await api.get<LogEntry[]>('/api/v1/logs/tail', { params });
    return response.data;
  },

  async startRun(req: StartRunRequest): Promise<StartRunResponse> {
    const response = await api.post<StartRunResponse>('/api/v1/runs/start', req);
    return response.data;
  },

  async stopRun(): Promise<StopRunResponse> {
    const response = await api.post<StopRunResponse>('/api/v1/runs/stop');
    return response.data;
  },

  async listRuns(): Promise<RunHistoryEntry[]> {
    const response = await api.get<RunHistoryEntry[]>('/api/v1/runs/list');
    return response.data;
  },

  async exportRun(runId: string): Promise<any> {
    const response = await api.get(`/api/v1/runs/${runId}/export`);
    return response.data;
  },
};
