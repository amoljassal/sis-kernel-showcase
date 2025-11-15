/**
 * Deployment management panel with phase timeline, metrics, and controls
 */

import { useState, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  deploymentApi,
  DeploymentStatus,
  PhaseTransition,
  DeploymentConfigRequest,
} from '../lib/api';
import {
  Rocket,
  ChevronRight,
  CheckCircle,
  AlertCircle,
  Clock,
  TrendingUp,
  TrendingDown,
  RotateCcw,
  Play,
  Settings,
  Activity,
} from 'lucide-react';

export function DeploymentPanel() {
  const queryClient = useQueryClient();
  const tableRef = useRef<HTMLDivElement>(null);
  const [autoAdvance, setAutoAdvance] = useState(true);
  const [autoRollback, setAutoRollback] = useState(true);
  const [errorThreshold, setErrorThreshold] = useState('5');
  const [rollbackReason, setRollbackReason] = useState('');

  // Fetch deployment status
  const { data: status, error: statusError } = useQuery({
    queryKey: ['deployment', 'status'],
    queryFn: () => deploymentApi.getStatus(),
    refetchInterval: 2000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  // Fetch phase transition history
  const { data: historyData } = useQuery({
    queryKey: ['deployment', 'history'],
    queryFn: () => deploymentApi.getHistory({ limit: 100 }),
    refetchInterval: 5000,
  });

  const transitions = historyData?.transitions ?? [];

  // Virtualizer for history table
  const rowVirtualizer = useVirtualizer({
    count: transitions.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 72,
    overscan: 10,
  });

  // Mutations
  const advanceMutation = useMutation({
    mutationFn: (force: boolean) => deploymentApi.advance(force),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['deployment'] });
    },
  });

  const rollbackMutation = useMutation({
    mutationFn: (reason: string) => deploymentApi.rollback(reason),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['deployment'] });
      setRollbackReason('');
    },
  });

  const updateConfigMutation = useMutation({
    mutationFn: (config: DeploymentConfigRequest) => deploymentApi.updateConfig(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['deployment'] });
    },
  });

  // Helper to get phase color
  const getPhaseColor = (phaseId: string) => {
    switch (phaseId) {
      case 'A': return 'blue';
      case 'B': return 'green';
      case 'C': return 'yellow';
      case 'D': return 'purple';
      default: return 'gray';
    }
  };

  // Helper to get trigger badge
  const getTriggerBadge = (trigger: PhaseTransition['trigger']) => {
    switch (trigger) {
      case 'auto_advance':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">
            <Play className="h-3 w-3" />
            Auto Advance
          </span>
        );
      case 'manual_advance':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
            <Play className="h-3 w-3" />
            Manual Advance
          </span>
        );
      case 'auto_rollback':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <RotateCcw className="h-3 w-3" />
            Auto Rollback
          </span>
        );
      case 'manual_rollback':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-orange-100 text-orange-800 rounded-full">
            <RotateCcw className="h-3 w-3" />
            Manual Rollback
          </span>
        );
    }
  };

  // Helper to format duration
  const formatDuration = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
    const hours = Math.floor(minutes / 60);
    return `${hours}h ${minutes % 60}m`;
  };

  // Handle config save
  const handleSaveConfig = () => {
    const threshold = parseInt(errorThreshold, 10);
    if (!isNaN(threshold) && threshold >= 0 && threshold <= 100) {
      updateConfigMutation.mutate({
        auto_advance_enabled: autoAdvance,
        auto_rollback_enabled: autoRollback,
        error_rate_threshold: threshold / 100,
      });
    }
  };

  if (statusError) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-red-800">
            <AlertCircle className="h-5 w-5" />
            <span className="font-medium">Failed to load deployment data</span>
          </div>
          <p className="mt-2 text-sm text-red-700">
            {statusError instanceof Error ? statusError.message : 'Unknown error'}
          </p>
        </div>
      </div>
    );
  }

  const currentPhase = status?.current_phase;

  return (
    <div className="h-full flex flex-col bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 p-6">
        <div className="flex items-center gap-3">
          <Rocket className="h-6 w-6 text-blue-600" />
          <h2 className="text-2xl font-bold text-gray-900">Deployment Management</h2>
        </div>
        <p className="mt-1 text-sm text-gray-600">
          Phased deployment with automatic rollback on errors
        </p>
      </div>

      {/* Current Phase Status */}
      {currentPhase && (
        <div className="p-6 bg-white border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Current Phase</h3>

          {/* Phase Timeline */}
          <div className="flex items-center gap-4 mb-6">
            {['A', 'B', 'C', 'D'].map((phase, idx) => {
              const isActive = phase === currentPhase.id;
              const isPast = phase < currentPhase.id;
              const color = getPhaseColor(phase);

              return (
                <div key={phase} className="flex items-center gap-2">
                  <div className="flex flex-col items-center">
                    <div
                      className={`w-12 h-12 rounded-full flex items-center justify-center text-lg font-bold border-2 ${
                        isActive
                          ? `bg-${color}-100 border-${color}-500 text-${color}-900`
                          : isPast
                          ? `bg-${color}-600 border-${color}-600 text-white`
                          : 'bg-gray-100 border-gray-300 text-gray-500'
                      }`}
                    >
                      {isPast ? <CheckCircle className="h-6 w-6" /> : phase}
                    </div>
                    <div className="mt-2 text-center">
                      <div className={`text-sm font-medium ${isActive ? `text-${color}-900` : 'text-gray-600'}`}>
                        Phase {phase}
                      </div>
                      {isActive && (
                        <div className="text-xs text-gray-500 mt-1">
                          {currentPhase.traffic_percentage}% traffic
                        </div>
                      )}
                    </div>
                  </div>
                  {idx < 3 && (
                    <ChevronRight
                      className={`h-6 w-6 ${
                        phase < currentPhase.id ? `text-${color}-600` : 'text-gray-300'
                      }`}
                    />
                  )}
                </div>
              );
            })}
          </div>

          {/* Phase Details */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="text-sm font-medium text-gray-900 mb-3">{currentPhase.name}</h4>
              <p className="text-sm text-gray-600 mb-4">{currentPhase.description}</p>

              <div className="space-y-2 text-sm">
                <div className="flex justify-between">
                  <span className="text-gray-600">Elapsed Time:</span>
                  <span className="font-medium text-gray-900">{formatDuration(currentPhase.elapsed_ms)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Min Duration:</span>
                  <span className="font-medium text-gray-900">{formatDuration(currentPhase.min_duration_ms)}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-600">Can Advance:</span>
                  <span className={`font-medium ${currentPhase.can_advance ? 'text-green-600' : 'text-red-600'}`}>
                    {currentPhase.can_advance ? 'Yes' : 'No'}
                  </span>
                </div>
              </div>
            </div>

            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="text-sm font-medium text-gray-900 mb-3">Phase Metrics</h4>

              <div className="space-y-3">
                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-600">Success Rate</span>
                    <span className="font-medium text-green-600">
                      {(currentPhase.success_rate * 100).toFixed(2)}%
                    </span>
                  </div>
                  <div className="bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-green-600 h-2 rounded-full"
                      style={{ width: `${currentPhase.success_rate * 100}%` }}
                    />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-600">Error Rate</span>
                    <span className="font-medium text-red-600">
                      {(currentPhase.error_rate * 100).toFixed(2)}%
                    </span>
                  </div>
                  <div className="bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-red-600 h-2 rounded-full"
                      style={{ width: `${currentPhase.error_rate * 100}%` }}
                    />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-600">Traffic</span>
                    <span className="font-medium text-blue-600">
                      {currentPhase.traffic_percentage}%
                    </span>
                  </div>
                  <div className="bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full"
                      style={{ width: `${currentPhase.traffic_percentage}%` }}
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Controls */}
      {status && (
        <div className="p-6 bg-white border-b border-gray-200">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Manual Controls */}
            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
                <Activity className="h-4 w-4" />
                Manual Controls
              </h4>

              <div className="space-y-3">
                <button
                  onClick={() => advanceMutation.mutate(false)}
                  disabled={!currentPhase?.can_advance || advanceMutation.isPending}
                  className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Play className="h-4 w-4" />
                  Advance to Next Phase
                </button>

                <button
                  onClick={() => advanceMutation.mutate(true)}
                  disabled={advanceMutation.isPending}
                  className="w-full px-4 py-2 bg-orange-600 text-white rounded-md hover:bg-orange-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Play className="h-4 w-4" />
                  Force Advance (Override)
                </button>

                <div className="space-y-2">
                  <input
                    type="text"
                    placeholder="Rollback reason..."
                    value={rollbackReason}
                    onChange={(e) => setRollbackReason(e.target.value)}
                    className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500"
                  />
                  <button
                    onClick={() => rollbackMutation.mutate(rollbackReason || 'Manual rollback')}
                    disabled={rollbackMutation.isPending}
                    className="w-full px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                  >
                    <RotateCcw className="h-4 w-4" />
                    Rollback to Previous Phase
                  </button>
                </div>
              </div>

              {status.rollback_count > 0 && (
                <div className="mt-3 text-xs text-orange-600">
                  Rollbacks: {status.rollback_count} / {status.max_rollbacks}
                </div>
              )}
            </div>

            {/* Configuration */}
            <div className="bg-gray-50 rounded-lg p-4">
              <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
                <Settings className="h-4 w-4" />
                Auto-Deployment Configuration
              </h4>

              <div className="space-y-3">
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={autoAdvance}
                    onChange={(e) => setAutoAdvance(e.target.checked)}
                    className="w-4 h-4 text-blue-600 border-gray-300 rounded focus:ring-blue-500"
                  />
                  <span className="text-sm text-gray-700">Auto-advance enabled</span>
                </label>

                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={autoRollback}
                    onChange={(e) => setAutoRollback(e.target.checked)}
                    className="w-4 h-4 text-red-600 border-gray-300 rounded focus:ring-red-500"
                  />
                  <span className="text-sm text-gray-700">Auto-rollback enabled</span>
                </label>

                <div>
                  <label htmlFor="error-threshold" className="block text-sm text-gray-700 mb-1">
                    Error Rate Threshold (%)
                  </label>
                  <input
                    id="error-threshold"
                    type="number"
                    min="0"
                    max="100"
                    value={errorThreshold}
                    onChange={(e) => setErrorThreshold(e.target.value)}
                    className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                <button
                  onClick={handleSaveConfig}
                  disabled={updateConfigMutation.isPending}
                  className="w-full px-4 py-2 bg-gray-700 text-white rounded-md hover:bg-gray-800 disabled:bg-gray-300 disabled:cursor-not-allowed"
                >
                  Save Configuration
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Phase Transition History */}
      <div className="flex-1 flex flex-col p-6 overflow-hidden">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Phase Transition History</h3>

        <div
          ref={tableRef}
          className="flex-1 overflow-auto bg-white border border-gray-200 rounded-lg"
          style={{ contain: 'strict' }}
        >
          <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const transition = transitions[virtualRow.index];
              const isRollback = transition.trigger.includes('rollback');

              return (
                <div
                  key={virtualRow.index}
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: `${virtualRow.size}px`,
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                  className="border-b border-gray-100 hover:bg-gray-50 px-4 py-3"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-3">
                      <div className="text-xs text-gray-500 w-24">
                        {new Date(transition.timestamp).toLocaleTimeString()}
                      </div>
                      {getTriggerBadge(transition.trigger)}
                      <div className="flex items-center gap-2 text-sm font-medium">
                        <span className="text-gray-900">Phase {transition.from_phase}</span>
                        {isRollback ? (
                          <TrendingDown className="h-4 w-4 text-red-600" />
                        ) : (
                          <TrendingUp className="h-4 w-4 text-green-600" />
                        )}
                        <span className="text-gray-900">Phase {transition.to_phase}</span>
                      </div>
                    </div>
                  </div>

                  <div className="ml-28 text-sm text-gray-600">
                    {transition.reason}
                  </div>

                  <div className="ml-28 mt-2 flex items-center gap-4 text-xs text-gray-500">
                    <span>Error Rate: {(transition.metrics_snapshot.error_rate * 100).toFixed(2)}%</span>
                    <span>Success Rate: {(transition.metrics_snapshot.success_rate * 100).toFixed(2)}%</span>
                    <span>Uptime: {transition.metrics_snapshot.uptime_hours.toFixed(1)}h</span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {transitions.length === 0 && (
          <div className="flex-1 flex items-center justify-center bg-white border border-gray-200 rounded-lg">
            <div className="text-center py-12">
              <Clock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">No phase transitions yet</p>
              <p className="text-sm text-gray-500 mt-1">
                Transition history will appear here as deployments progress through phases
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
