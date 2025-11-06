/**
 * Autonomy control panel with status, controls, and decisions dashboard
 */

import { useState, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  autonomyApi,
  AutonomyStatus,
  AutonomyDecision,
} from '../lib/api';
import {
  Power,
  PowerOff,
  RotateCcw,
  Clock,
  Target,
  TrendingUp,
  AlertTriangle,
  Check,
  X,
  Info,
} from 'lucide-react';

interface AutonomyPanelProps {
  onExplainDecision?: (decision: AutonomyDecision) => void;
}

export function AutonomyPanel({ onExplainDecision }: AutonomyPanelProps) {
  const queryClient = useQueryClient();
  const [intervalInput, setIntervalInput] = useState<string>('1000');
  const [thresholdInput, setThresholdInput] = useState<string>('500');
  const [selectedRows, _setSelectedRows] = useState<Set<string>>(new Set());
  const tableRef = useRef<HTMLDivElement>(null);

  // Fetch autonomy status
  const { data: status, error: statusError } = useQuery({
    queryKey: ['autonomy', 'status'],
    queryFn: () => autonomyApi.status(),
    refetchInterval: 2000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  // Fetch audit log (last 1000 decisions)
  const { data: decisions = [] } = useQuery({
    queryKey: ['autonomy', 'audit'],
    queryFn: () => autonomyApi.audit(1000),
    refetchInterval: 5000,
    enabled: status?.enabled === true,
  });

  // Mutations
  const turnOn = useMutation({
    mutationFn: () => autonomyApi.turnOn(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['autonomy'] }),
  });

  const turnOff = useMutation({
    mutationFn: () => autonomyApi.turnOff(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['autonomy'] }),
  });

  const reset = useMutation({
    mutationFn: () => autonomyApi.reset(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['autonomy'] }),
  });

  const setInterval = useMutation({
    mutationFn: (ms: number) => autonomyApi.setInterval(ms),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['autonomy'] }),
  });

  const setThreshold = useMutation({
    mutationFn: (threshold: number) => autonomyApi.setThreshold(threshold),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['autonomy'] }),
  });

  // Virtualizer for decisions table
  const rowVirtualizer = useVirtualizer({
    count: decisions.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 48,
    overscan: 10,
  });

  // Handle control actions
  const handleIntervalSubmit = () => {
    const ms = parseInt(intervalInput, 10);
    if (!isNaN(ms) && ms > 0) {
      setInterval.mutate(ms);
    }
  };

  const handleThresholdSubmit = () => {
    const threshold = parseInt(thresholdInput, 10);
    if (!isNaN(threshold) && threshold >= 0 && threshold <= 1000) {
      setThreshold.mutate(threshold);
    }
  };

  // Mode badge color
  const getModeColor = (mode: string) => {
    switch (mode) {
      case 'active':
        return 'text-green-500 bg-green-500/10';
      case 'safe_mode':
        return 'text-yellow-500 bg-yellow-500/10';
      case 'learning_frozen':
        return 'text-blue-500 bg-blue-500/10';
      default:
        return 'text-muted-foreground bg-muted';
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="p-4 border-b">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <TrendingUp className="h-5 w-5" />
          Autonomy Control
        </h3>
      </div>

      {/* Controls */}
      <div className="p-4 border-b space-y-3">
        <div className="flex gap-2">
          <button
            onClick={() => turnOn.mutate()}
            disabled={turnOn.isPending || status?.enabled}
            className="px-3 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm"
          >
            <Power className="h-4 w-4" />
            On
          </button>
          <button
            onClick={() => turnOff.mutate()}
            disabled={turnOff.isPending || !status?.enabled}
            className="px-3 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm"
          >
            <PowerOff className="h-4 w-4" />
            Off
          </button>
          <button
            onClick={() => reset.mutate()}
            disabled={reset.isPending}
            className="px-3 py-2 bg-muted text-foreground rounded-md hover:bg-muted/80 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2 text-sm"
          >
            <RotateCcw className="h-4 w-4" />
            Reset
          </button>
        </div>

        <div className="grid grid-cols-2 gap-3">
          {/* Interval */}
          <div>
            <label className="text-xs text-muted-foreground flex items-center gap-1 mb-1">
              <Clock className="h-3 w-3" />
              Interval (ms)
            </label>
            <div className="flex gap-1">
              <input
                type="number"
                value={intervalInput}
                onChange={(e) => setIntervalInput(e.target.value)}
                className="flex-1 px-2 py-1 bg-background border rounded text-sm"
                min="100"
                step="100"
              />
              <button
                onClick={handleIntervalSubmit}
                disabled={setInterval.isPending}
                className="px-2 py-1 bg-primary text-primary-foreground rounded text-sm hover:bg-primary/90"
              >
                Set
              </button>
            </div>
          </div>

          {/* Confidence Threshold */}
          <div>
            <label className="text-xs text-muted-foreground flex items-center gap-1 mb-1">
              <Target className="h-3 w-3" />
              Conf Threshold (0-1000)
            </label>
            <div className="flex gap-1">
              <input
                type="range"
                value={thresholdInput}
                onChange={(e) => setThresholdInput(e.target.value)}
                className="flex-1"
                min="0"
                max="1000"
                step="10"
              />
              <input
                type="number"
                value={thresholdInput}
                onChange={(e) => setThresholdInput(e.target.value)}
                className="w-16 px-2 py-1 bg-background border rounded text-sm"
                min="0"
                max="1000"
              />
              <button
                onClick={handleThresholdSubmit}
                disabled={setThreshold.isPending}
                className="px-2 py-1 bg-primary text-primary-foreground rounded text-sm hover:bg-primary/90"
              >
                Set
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Status Cards */}
      {status && (
        <div className="p-4 border-b">
          <div className="grid grid-cols-4 gap-2 mb-3">
            {/* Enabled */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Status</div>
              <div className={`text-sm font-semibold ${status.enabled ? 'text-green-500' : 'text-muted-foreground'}`}>
                {status.enabled ? 'Enabled' : 'Disabled'}
              </div>
            </div>

            {/* Mode */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Mode</div>
              <div className={`text-sm font-semibold px-2 py-0.5 rounded inline-block ${getModeColor(status.mode)}`}>
                {status.mode}
              </div>
            </div>

            {/* Interval */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Interval</div>
              <div className="text-sm font-semibold">{status.interval_ms}ms</div>
            </div>

            {/* Threshold */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Threshold</div>
              <div className="text-sm font-semibold">{status.conf_threshold}</div>
            </div>
          </div>

          <div className="grid grid-cols-4 gap-2">
            {/* Total Decisions */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Total</div>
              <div className="text-sm font-semibold">{status.total_decisions}</div>
            </div>

            {/* Accepted */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
                <Check className="h-3 w-3" />
                Accepted
              </div>
              <div className="text-sm font-semibold text-green-500">{status.accepted}</div>
            </div>

            {/* Deferred */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
                <X className="h-3 w-3" />
                Deferred
              </div>
              <div className="text-sm font-semibold text-yellow-500">{status.deferred}</div>
            </div>

            {/* Watchdog */}
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
                <AlertTriangle className="h-3 w-3" />
                Watchdog
              </div>
              <div className="text-sm font-semibold text-red-500">{status.watchdog_resets}</div>
            </div>
          </div>
        </div>
      )}

      {/* Error Display */}
      {statusError && (
        <div className="mx-4 my-2 p-3 bg-red-500/10 border border-red-500 rounded text-sm">
          <div className="flex items-start gap-2">
            <AlertTriangle className="h-4 w-4 text-red-500 mt-0.5" />
            <div>
              <div className="font-semibold text-red-500">Failed to load status</div>
              <div className="text-red-500/80 text-xs mt-1">
                {statusError instanceof Error ? statusError.message : 'Unknown error'}
              </div>
              <button
                onClick={() => queryClient.invalidateQueries({ queryKey: ['autonomy', 'status'] })}
                className="mt-2 text-xs text-red-500 underline hover:no-underline"
              >
                Retry
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Decisions Table */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="px-4 py-2 bg-muted/50 border-b">
          <div className="text-sm font-semibold">
            Recent Decisions ({decisions.length})
          </div>
        </div>

        <div
          ref={tableRef}
          className="flex-1 overflow-y-auto"
          style={{ contain: 'strict' }}
        >
          {decisions.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
              {status?.enabled ? 'No decisions yet' : 'Enable autonomy to see decisions'}
            </div>
          ) : (
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const decision = decisions[virtualRow.index];
                const isSelected = selectedRows.has(decision.id);

                return (
                  <div
                    key={virtualRow.key}
                    onClick={() => onExplainDecision?.(decision)}
                    className={`absolute top-0 left-0 w-full px-4 py-2 cursor-pointer transition-colors border-b ${
                      isSelected
                        ? 'bg-primary/10 border-primary'
                        : 'hover:bg-muted/50'
                    }`}
                    style={{
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <div className="flex items-center justify-between text-xs">
                      <div className="flex items-center gap-4 flex-1">
                        {/* ID */}
                        <div className="w-20 truncate font-mono">
                          {decision.id.substring(0, 8)}...
                        </div>

                        {/* Timestamp */}
                        <div className="w-24 text-muted-foreground">
                          {new Date(decision.timestamp).toLocaleTimeString()}
                        </div>

                        {/* Action */}
                        <div className="flex-1 truncate">
                          {decision.action}
                        </div>

                        {/* Confidence */}
                        <div className="w-16 text-right font-semibold">
                          {(decision.confidence * 100).toFixed(0)}%
                        </div>

                        {/* Reward */}
                        {decision.reward !== undefined && (
                          <div className={`w-16 text-right ${decision.reward > 0 ? 'text-green-500' : decision.reward < 0 ? 'text-red-500' : 'text-muted-foreground'}`}>
                            {decision.reward > 0 ? '+' : ''}{decision.reward.toFixed(2)}
                          </div>
                        )}

                        {/* Executed */}
                        <div className="w-16">
                          {decision.executed ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <X className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>

                        {/* Explain Icon */}
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            onExplainDecision?.(decision);
                          }}
                          className="p-1 hover:bg-primary/20 rounded"
                        >
                          <Info className="h-4 w-4" />
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
