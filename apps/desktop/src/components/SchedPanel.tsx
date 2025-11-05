/**
 * Scheduling Panel - Workload scheduling and feature management
 */

import { useState, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { schedApi, type Workload } from '../lib/api';
import { Cpu, Settings, AlertCircle, RefreshCw } from 'lucide-react';

export function SchedPanel() {
  const queryClient = useQueryClient();

  // State
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [newPriority, setNewPriority] = useState<string>('100');
  const [affinityMask, setAffinityMask] = useState<string>('0xF');
  const [error, setError] = useState<string>('');

  // Virtualization ref
  const tableRef = useRef<HTMLDivElement>(null);

  // Fetch workloads
  const { data: workloads = [] } = useQuery({
    queryKey: ['sched', 'workloads'],
    queryFn: () => schedApi.workloads(),
    refetchInterval: 3000,
    retry: 2,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 10000),
  });

  // Fetch circuit breaker status
  const { data: circuitBreaker } = useQuery({
    queryKey: ['sched', 'circuit-breaker'],
    queryFn: () => schedApi.circuitBreakerStatus(),
    refetchInterval: 5000,
    retry: 1,
  });

  // Set priority mutation
  const setPriority = useMutation({
    mutationFn: (req: { pid: number; prio: number }) => schedApi.setPriority(req),
    onSuccess: () => {
      setError('');
      queryClient.invalidateQueries({ queryKey: ['sched', 'workloads'] });
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to set priority');
    },
  });

  // Set affinity mutation
  const setAffinity = useMutation({
    mutationFn: (req: { pid: number; cpuMask: string }) => schedApi.setAffinity(req),
    onSuccess: () => {
      setError('');
      queryClient.invalidateQueries({ queryKey: ['sched', 'workloads'] });
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to set affinity');
    },
  });

  // Toggle feature mutation
  const toggleFeature = useMutation({
    mutationFn: (req: { name: string; enable: boolean }) => schedApi.setFeature(req),
    onSuccess: () => {
      setError('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to toggle feature');
    },
  });

  // Reset circuit breaker mutation
  const resetCircuitBreaker = useMutation({
    mutationFn: () => schedApi.circuitBreakerReset(),
    onSuccess: () => {
      setError('');
      queryClient.invalidateQueries({ queryKey: ['sched', 'circuit-breaker'] });
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to reset circuit breaker');
    },
  });

  // Virtualized workloads table
  const rowVirtualizer = useVirtualizer({
    count: workloads.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 56,
    overscan: 10,
  });

  // Circuit breaker state colors
  const getCircuitBreakerColor = (state: string) => {
    switch (state) {
      case 'Closed':
        return 'text-green-500 bg-green-500/10';
      case 'Open':
        return 'text-red-500 bg-red-500/10';
      case 'HalfOpen':
        return 'text-yellow-500 bg-yellow-500/10';
      default:
        return 'text-muted-foreground bg-muted-foreground/10';
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-4 gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Cpu className="h-6 w-6" />
          Scheduling
        </h2>
        <span className="text-sm text-muted-foreground">
          {workloads.length} workloads
        </span>
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-destructive/10 text-destructive px-4 py-3 rounded-md flex items-start gap-2">
          <AlertCircle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-semibold">Error</p>
            <p className="text-sm">{error}</p>
          </div>
          <button onClick={() => setError('')} className="text-sm hover:underline">
            Dismiss
          </button>
        </div>
      )}

      {/* Controls Grid */}
      <div className="grid grid-cols-3 gap-4">
        {/* Priority Control */}
        <div className="bg-muted p-4 rounded-md space-y-2">
          <h3 className="font-semibold text-sm">Change Priority</h3>
          <div className="space-y-2">
            <input
              type="number"
              value={newPriority}
              onChange={(e) => setNewPriority(e.target.value)}
              placeholder="Priority (0-255)"
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            />
            <button
              onClick={() => {
                if (selectedPid !== null) {
                  setPriority.mutate({ pid: selectedPid, prio: parseInt(newPriority) });
                }
              }}
              disabled={setPriority.isPending || selectedPid === null}
              className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm"
            >
              {setPriority.isPending ? 'Setting...' : 'Set Priority'}
            </button>
            {selectedPid !== null && (
              <p className="text-xs text-muted-foreground">PID: {selectedPid}</p>
            )}
          </div>
        </div>

        {/* Affinity Control */}
        <div className="bg-muted p-4 rounded-md space-y-2">
          <h3 className="font-semibold text-sm">CPU Affinity</h3>
          <div className="space-y-2">
            <input
              type="text"
              value={affinityMask}
              onChange={(e) => setAffinityMask(e.target.value)}
              placeholder="CPU Mask (e.g., 0xF)"
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            />
            <button
              onClick={() => {
                if (selectedPid !== null) {
                  setAffinity.mutate({ pid: selectedPid, cpuMask: affinityMask });
                }
              }}
              disabled={setAffinity.isPending || selectedPid === null}
              className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm"
            >
              {setAffinity.isPending ? 'Setting...' : 'Set Affinity'}
            </button>
          </div>
        </div>

        {/* Circuit Breaker */}
        <div className="bg-muted p-4 rounded-md space-y-2">
          <h3 className="font-semibold text-sm flex items-center gap-2">
            <Settings className="h-4 w-4" />
            Circuit Breaker
          </h3>
          {circuitBreaker && (
            <div className="space-y-2">
              <div className={`px-3 py-2 rounded-md ${getCircuitBreakerColor(circuitBreaker.state)}`}>
                <p className="font-semibold text-sm">{circuitBreaker.state}</p>
              </div>
              <div className="text-xs text-muted-foreground space-y-1">
                <p>Failures: {circuitBreaker.consecutive_failures}/{circuitBreaker.failure_threshold}</p>
                <p>Timeout: {(circuitBreaker.reset_timeout_us / 1000).toFixed(1)}ms</p>
              </div>
              <button
                onClick={() => resetCircuitBreaker.mutate()}
                disabled={resetCircuitBreaker.isPending}
                className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm flex items-center justify-center gap-2"
              >
                <RefreshCw className="h-4 w-4" />
                {resetCircuitBreaker.isPending ? 'Resetting...' : 'Reset'}
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Feature Toggles */}
      <div className="bg-muted p-4 rounded-md">
        <h3 className="font-semibold mb-3 text-sm">Feature Toggles</h3>
        <div className="grid grid-cols-3 gap-4">
          <button
            onClick={() => toggleFeature.mutate({ name: 'autonomous-scheduling', enable: true })}
            disabled={toggleFeature.isPending}
            className="px-3 py-2 bg-green-500/20 text-green-500 rounded-md hover:bg-green-500/30 disabled:opacity-50 text-sm"
          >
            Enable Autonomous
          </button>
          <button
            onClick={() => toggleFeature.mutate({ name: 'autonomous-scheduling', enable: false })}
            disabled={toggleFeature.isPending}
            className="px-3 py-2 bg-red-500/20 text-red-500 rounded-md hover:bg-red-500/30 disabled:opacity-50 text-sm"
          >
            Disable Autonomous
          </button>
          <button
            onClick={() => toggleFeature.mutate({ name: 'shadow-mode', enable: true })}
            disabled={toggleFeature.isPending}
            className="px-3 py-2 bg-blue-500/20 text-blue-500 rounded-md hover:bg-blue-500/30 disabled:opacity-50 text-sm"
          >
            Enable Shadow Mode
          </button>
        </div>
      </div>

      {/* Workloads Table */}
      <div className="flex-1 bg-muted rounded-md flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b border-border flex items-center justify-between">
          <h3 className="font-semibold">Workloads</h3>
          <div className="flex gap-4 text-sm text-muted-foreground">
            <span>PID</span>
            <span>Name</span>
            <span>Priority</span>
            <span>CPU</span>
            <span>State</span>
          </div>
        </div>
        <div
          ref={tableRef}
          className="flex-1 overflow-y-auto"
          style={{ contain: 'strict' }}
        >
          {workloads.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              No workloads available
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
                const workload = workloads[virtualRow.index];
                const isSelected = selectedPid === workload.pid;

                return (
                  <div
                    key={virtualRow.key}
                    style={{
                      position: 'absolute',
                      top: 0,
                      left: 0,
                      width: '100%',
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                    className={`px-4 py-3 border-b border-border cursor-pointer transition-colors ${
                      isSelected
                        ? 'bg-primary/20'
                        : 'hover:bg-muted-foreground/5'
                    }`}
                    onClick={() => setSelectedPid(workload.pid)}
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4">
                        <span className="font-mono text-sm font-semibold w-16">
                          {workload.pid}
                        </span>
                        <span className="text-sm min-w-[200px]">
                          {workload.name}
                        </span>
                      </div>
                      <div className="flex items-center gap-8 text-sm text-muted-foreground">
                        <span className="w-12 text-center">
                          {workload.prio}
                        </span>
                        <span className="w-12 text-center">
                          CPU {workload.cpu}
                        </span>
                        <span className={`w-24 text-center px-2 py-1 rounded ${
                          workload.state === 'Running'
                            ? 'bg-green-500/10 text-green-500'
                            : workload.state === 'Sleeping'
                            ? 'bg-blue-500/10 text-blue-500'
                            : 'bg-muted-foreground/10'
                        }`}>
                          {workload.state}
                        </span>
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
