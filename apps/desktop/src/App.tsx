/**
 * Main application component
 */

import { useEffect, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { qemuApi, QemuConfig, QemuState, QemuStatus } from './lib/api';
import { useWebSocket } from './lib/useWebSocket';
import { writeToTerminal } from './components/Terminal';
import { StatusBadge } from './components/StatusBadge';
import { Terminal } from './components/Terminal';
import { BootMarkers } from './components/BootMarkers';
import { QemuProfileSelector } from './components/QemuProfileSelector';
import { MetricsSparkline } from './components/MetricsSparkline';
import { ShellCommandInput } from './components/ShellCommandInput';
import { SelfCheckRunner } from './components/SelfCheckRunner';
import { ReplayControls } from './components/ReplayControls';
import { AlertCircle, Activity, Terminal as TerminalIcon } from 'lucide-react';
import './App.css';

interface MetricPoint {
  timestamp: number;
  value: number;
}

function App() {
  const queryClient = useQueryClient();
  const [daemonHealthy, setDaemonHealthy] = useState(false);
  const [bootMarkers, setBootMarkers] = useState<Record<string, boolean>>({});
  const [metrics, setMetrics] = useState<Record<string, MetricPoint[]>>({});

  // Check daemon health
  const { data: daemonStatus } = useQuery({
    queryKey: ['daemon-health'],
    queryFn: async () => {
      const status = await invoke<{ healthy: boolean; version?: string }>(
        'check_daemon'
      );
      setDaemonHealthy(status.healthy);
      return status;
    },
    refetchInterval: 5000,
    retry: 2,
  });

  // Launch daemon if not running
  const launchDaemon = useMutation({
    mutationFn: async () => {
      return await invoke<string>('launch_daemon');
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['daemon-health'] });
      queryClient.invalidateQueries({ queryKey: ['qemu-status'] });
    },
  });

  // Get QEMU status
  const { data: qemuStatus } = useQuery<QemuStatus>({
    queryKey: ['qemu-status'],
    queryFn: () => qemuApi.status(),
    enabled: daemonHealthy,
    refetchInterval: 1000,
  });

  // Run QEMU
  const runQemu = useMutation({
    mutationFn: (config: QemuConfig) => qemuApi.run(config),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qemu-status'] });
      setBootMarkers({});
      setMetrics({});
    },
  });

  // Stop QEMU
  const stopQemu = useMutation({
    mutationFn: () => qemuApi.stop(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['qemu-status'] });
    },
  });

  // WebSocket event handling
  useWebSocket((event) => {
    // Handle raw lines (terminal output)
    if (event.type === 'raw_line') {
      writeToTerminal(event.line + '\r\n');
    }

    // Handle parsed events
    if (event.type === 'parsed') {
      const parsed = event.event;

      // Update boot markers
      if (parsed.type === 'marker') {
        setBootMarkers((prev) => ({
          ...prev,
          [parsed.marker]: true,
        }));
      }

      // Update metrics
      if (parsed.type === 'metric') {
        setMetrics((prev) => ({
          ...prev,
          [parsed.name]: [
            ...(prev[parsed.name] || []),
            {
              timestamp: parsed.timestamp,
              value: parsed.value,
            },
          ],
        }));
      }
    }

    // Handle state changes
    if (event.type === 'state_changed') {
      queryClient.invalidateQueries({ queryKey: ['qemu-status'] });
    }
  });

  const currentState = qemuStatus?.state || QemuState.Idle;

  return (
    <div className="h-screen flex flex-col bg-background text-foreground">
      {/* Header */}
      <header className="border-b px-6 py-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Activity className="h-6 w-6 text-primary" />
            <h1 className="text-2xl font-bold">SIS Kernel</h1>
          </div>

          <div className="flex items-center gap-4">
            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">Daemon:</span>
              <StatusBadge
                state={daemonHealthy ? QemuState.Running : 'disconnected'}
              />
            </div>

            {!daemonHealthy && (
              <button
                onClick={() => launchDaemon.mutate()}
                disabled={launchDaemon.isPending}
                className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm"
              >
                {launchDaemon.isPending ? 'Launching...' : 'Launch Daemon'}
              </button>
            )}

            <div className="flex items-center gap-2">
              <span className="text-sm text-muted-foreground">QEMU:</span>
              <StatusBadge state={currentState} />
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        {!daemonHealthy ? (
          <div className="h-full flex items-center justify-center">
            <div className="text-center max-w-md">
              <AlertCircle className="h-16 w-16 text-destructive mx-auto mb-4" />
              <h2 className="text-2xl font-bold mb-2">Daemon Not Running</h2>
              <p className="text-muted-foreground mb-4">
                The sisctl daemon is required to manage QEMU instances. Please
                launch the daemon to continue.
              </p>
              <button
                onClick={() => launchDaemon.mutate()}
                disabled={launchDaemon.isPending}
                className="px-6 py-3 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
              >
                {launchDaemon.isPending ? 'Launching...' : 'Launch Daemon'}
              </button>
            </div>
          </div>
        ) : (
          <div className="h-full grid grid-cols-3 gap-4 p-4">
            {/* Left Column - Profile & Boot Status */}
            <div className="space-y-4 overflow-y-auto">
              <QemuProfileSelector
                onRun={(config) => runQemu.mutate(config)}
                onStop={() => stopQemu.mutate()}
                qemuState={currentState}
              />

              <BootMarkers markers={bootMarkers} />

              <ShellCommandInput
                disabled={currentState !== QemuState.Running}
              />

              <SelfCheckRunner
                disabled={currentState !== QemuState.Running}
              />

              <ReplayControls
                disabled={currentState !== QemuState.Idle}
              />
            </div>

            {/* Middle Column - Terminal */}
            <div className="bg-card rounded-lg border p-4 flex flex-col">
              <div className="flex items-center gap-2 mb-4">
                <TerminalIcon className="h-5 w-5" />
                <h3 className="text-lg font-semibold">Terminal</h3>
              </div>
              <div className="flex-1 min-h-0">
                <Terminal />
              </div>
            </div>

            {/* Right Column - Metrics */}
            <div className="space-y-4 overflow-y-auto">
              <h3 className="text-lg font-semibold px-4">Live Metrics</h3>

              {Object.keys(metrics).length === 0 ? (
                <div className="bg-card rounded-lg border p-8 text-center text-muted-foreground">
                  No metrics yet. Start QEMU to see live data.
                </div>
              ) : (
                Object.entries(metrics).map(([name, data]) => (
                  <MetricsSparkline
                    key={name}
                    title={name}
                    metrics={data}
                    color={
                      name.includes('latency')
                        ? '#ef4444'
                        : name.includes('memory')
                        ? '#3b82f6'
                        : '#10b981'
                    }
                  />
                ))
              )}
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="border-t px-6 py-2 text-sm text-muted-foreground">
        <div className="flex items-center justify-between">
          <div>
            {qemuStatus && (
              <>
                Lines: {qemuStatus.lines_processed} | Events:{' '}
                {qemuStatus.events_emitted}
              </>
            )}
          </div>
          <div>
            {daemonStatus?.version && `Daemon v${daemonStatus.version}`}
          </div>
        </div>
      </footer>
    </div>
  );
}

export default App;
