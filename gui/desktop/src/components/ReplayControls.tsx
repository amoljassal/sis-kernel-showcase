/**
 * Replay Controls Component
 *
 * UI for controlling replay functionality:
 * - Speed selector (Instant/Fast/RealTime)
 * - Sample selector
 * - Start/Stop buttons with proper state management
 * - Progress bar (0-100%)
 * - Error display
 */

import { useState } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { replayApi, ReplayState, ReplayStatus } from '../lib/api';
import { Play, Square, Zap, FastForward, Clock, AlertCircle } from 'lucide-react';

interface ReplayControlsProps {
  disabled?: boolean;
}

const SPEED_OPTIONS = [
  { value: 'instant', label: 'Instant', icon: Zap },
  { value: 'fast', label: 'Fast', icon: FastForward },
  { value: 'realtime', label: 'Real-time', icon: Clock },
];

const SAMPLE_OPTIONS = [
  { value: 'boot_minimal', label: 'Boot (Minimal)' },
  { value: 'boot_with_metrics', label: 'Boot (With Metrics)' },
  { value: 'self_check', label: 'Self-Check' },
];

export function ReplayControls({ disabled = false }: ReplayControlsProps) {
  const queryClient = useQueryClient();
  const [speed, setSpeed] = useState('realtime');
  const [sample, setSample] = useState('boot_minimal');
  const [error, setError] = useState<string | null>(null);

  // Query replay status
  const { data: status } = useQuery<ReplayStatus>({
    queryKey: ['replay-status'],
    queryFn: () => replayApi.status(),
    refetchInterval: 2000,
  });

  // Start replay mutation
  const startReplay = useMutation({
    mutationFn: () =>
      replayApi.start({
        mode: 'sample',
        logSource: sample,
        speed,
      }),
    onSuccess: () => {
      setError(null);
      queryClient.invalidateQueries({ queryKey: ['replay-status'] });
    },
    onError: (err: any) => {
      const message = err.response?.data?.detail || err.message || 'Failed to start replay';
      setError(message);
    },
  });

  // Stop replay mutation
  const stopReplay = useMutation({
    mutationFn: () => replayApi.stop(),
    onSuccess: () => {
      setError(null);
      queryClient.invalidateQueries({ queryKey: ['replay-status'] });
    },
    onError: (err: any) => {
      const message = err.response?.data?.detail || err.message || 'Failed to stop replay';
      setError(message);
    },
  });

  const isRunning = status?.state === ReplayState.Running;
  const isDisabled = disabled || startReplay.isPending || stopReplay.isPending;

  return (
    <div className="bg-card rounded-lg border p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold">Replay Controls</h3>
        <div className="flex items-center gap-2">
          <span className="text-xs text-muted-foreground">Mode:</span>
          <span className={`text-xs font-medium px-2 py-1 rounded ${
            isRunning
              ? 'bg-green-500/20 text-green-500'
              : 'bg-muted text-muted-foreground'
          }`}>
            {isRunning ? 'Running' : 'Idle'}
          </span>
        </div>
      </div>

      {/* Speed Selector */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-muted-foreground">
          Speed
        </label>
        <div className="flex gap-2">
          {SPEED_OPTIONS.map((option) => {
            const Icon = option.icon;
            return (
              <button
                key={option.value}
                onClick={() => setSpeed(option.value)}
                disabled={isRunning || isDisabled}
                className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-md border text-sm transition-colors ${
                  speed === option.value
                    ? 'bg-primary text-primary-foreground border-primary'
                    : 'bg-background hover:bg-accent hover:text-accent-foreground'
                } disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                <Icon className="h-4 w-4" />
                {option.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Sample Selector */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-muted-foreground">
          Sample Log
        </label>
        <select
          value={sample}
          onChange={(e) => setSample(e.target.value)}
          disabled={isRunning || isDisabled}
          className="w-full px-3 py-2 bg-background border rounded-md text-sm disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {SAMPLE_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      {/* Progress Bar */}
      {isRunning && (
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Progress</span>
            <span className="font-medium">{status?.progress || 0}%</span>
          </div>
          <div className="h-2 bg-muted rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all duration-300"
              style={{ width: `${status?.progress || 0}%` }}
            />
          </div>
          {status?.source && (
            <p className="text-xs text-muted-foreground">
              Source: {status.source}
            </p>
          )}
        </div>
      )}

      {/* Start/Stop Buttons */}
      <div className="flex gap-2">
        {!isRunning ? (
          <button
            onClick={() => startReplay.mutate()}
            disabled={isDisabled}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Play className="h-4 w-4" />
            {startReplay.isPending ? 'Starting...' : 'Start Replay'}
          </button>
        ) : (
          <button
            onClick={() => stopReplay.mutate()}
            disabled={stopReplay.isPending}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            <Square className="h-4 w-4" />
            {stopReplay.isPending ? 'Stopping...' : 'Stop Replay'}
          </button>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="flex items-start gap-2 p-3 bg-destructive/10 border border-destructive/20 rounded-md text-sm">
          <AlertCircle className="h-4 w-4 text-destructive mt-0.5 flex-shrink-0" />
          <div className="flex-1">
            <p className="font-medium text-destructive">Error</p>
            <p className="text-destructive/80 mt-1">{error}</p>
          </div>
          <button
            onClick={() => setError(null)}
            className="text-destructive/60 hover:text-destructive"
          >
            ×
          </button>
        </div>
      )}
    </div>
  );
}
