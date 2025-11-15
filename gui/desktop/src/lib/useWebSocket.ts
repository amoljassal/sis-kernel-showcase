/**
 * WebSocket hook for receiving events from daemon
 */

import { useEffect, useRef, useState } from 'react';

// Default to IPv4 loopback to avoid IPv6 (::1) pitfalls
const WS_URL = import.meta.env.VITE_WS_URL || 'ws://127.0.0.1:8871/events';

// M4 WS Event Types
export interface GraphStateEvent {
  type: 'graph_state';
  graphId: string;
  state: {
    operators: Array<{
      id: string;
      name?: string;
      prio?: number;
      stage?: string;
      stats?: { execCount: number; avgUs: number };
    }>;
    channels: Array<{
      id: string;
      cap: number;
      depth?: number;
    }>;
  };
  ts: number;
}

export interface SchedEvent {
  type: 'sched_event';
  event: 'prio_change' | 'affinity_change' | 'feature_toggle';
  payload: any;
  ts: number;
}

export interface LlmTokensEvent {
  type: 'llm_tokens';
  requestId: string;
  chunk: string;
  done: boolean;
  ts: number;
}

export interface LogLineEvent {
  type: 'log_line';
  level: 'debug' | 'info' | 'warn' | 'error';
  source: 'daemon' | 'qemu' | 'kernel';
  msg: string;
  ts: number;
}

// Phase 2 Event Types
export interface OrchestrationDecisionEvent {
  type: 'orchestration_decision';
  decision_type: string;
  action: string;
  confidence?: number;
  agents: string[];
  latency_us: number;
  description: string;
  ts: number;
}

export interface ConflictResolvedEvent {
  type: 'conflict_resolved';
  conflict_id: string;
  agents: string[];
  resolution: string;
  resolution_time_us: number;
  ts: number;
}

export interface PhaseTransitionEvent {
  type: 'phase_transition';
  from_phase: string;
  to_phase: string;
  trigger: 'auto_advance' | 'auto_rollback' | 'manual_advance' | 'manual_rollback';
  reason: string;
  metrics_snapshot: {
    error_rate: number;
    success_rate: number;
    uptime_hours: number;
  };
  ts: number;
}

export interface DriftAlertEvent {
  type: 'drift_alert';
  drift_level: 'normal' | 'warning' | 'critical';
  baseline_accuracy: number;
  current_accuracy: number;
  accuracy_delta: number;
  sample_count: number;
  auto_retrain_triggered: boolean;
  message: string;
  ts: number;
}

export interface VersionCommitEvent {
  type: 'version_commit';
  version_id: number;
  parent_version?: number;
  description: string;
  metadata: any;
  ts: number;
}

export type WebSocketEvent =
  | { type: 'state_changed'; [key: string]: any }
  | { type: 'parsed'; [key: string]: any }
  | { type: 'raw_line'; [key: string]: any }
  | { type: 'metric_batch'; points: any[]; seq?: number; dropped_count?: number }
  | GraphStateEvent
  | SchedEvent
  | LlmTokensEvent
  | LogLineEvent
  | OrchestrationDecisionEvent
  | ConflictResolvedEvent
  | PhaseTransitionEvent
  | DriftAlertEvent
  | VersionCommitEvent;

export function useWebSocket(onEvent?: (event: WebSocketEvent) => void) {
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = () => {
    try {
      const ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        setError(null);
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onEvent?.(data);
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      ws.onerror = (event) => {
        console.error('WebSocket error:', event);
        setError('WebSocket connection error');
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);

        // Reconnect after 3 seconds
        reconnectTimeoutRef.current = setTimeout(() => {
          console.log('Reconnecting WebSocket...');
          connect();
        }, 3000);
      };

      wsRef.current = ws;
    } catch (e) {
      console.error('Failed to create WebSocket:', e);
      setError('Failed to connect to daemon');
    }
  };

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  return { isConnected, error };
}
