/**
 * Main application component
 */

import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/tauri';
import { qemuApi, QemuConfig, QemuState, QemuStatus, AutonomyDecision } from './lib/api';
import { useWebSocket } from './lib/useWebSocket';
import { writeToTerminal } from './components/Terminal';
import { StatusBadge } from './components/StatusBadge';
import { Terminal } from './components/Terminal';
import { BootMarkers } from './components/BootMarkers';
import { QemuProfileSelector } from './components/QemuProfileSelector';
import { MetricsPanel } from './components/MetricsPanel';
import { Dashboard } from './components/Dashboard';
import { AutonomyPanel } from './components/AutonomyPanel';
import { ApprovalsPanel } from './components/ApprovalsPanel';
import { ExplainView } from './components/ExplainView';
import { WhatIfSimulator } from './components/WhatIfSimulator';
import { ShellCommandInput } from './components/ShellCommandInput';
import { SelfCheckRunner } from './components/SelfCheckRunner';
import { ReplayControls } from './components/ReplayControls';
import { GraphPanel } from './components/GraphPanel';
import { SchedPanel } from './components/SchedPanel';
import { LlmPanel } from './components/LlmPanel';
import { LogsPanel } from './components/LogsPanel';
import { CrashPanel } from './components/CrashPanel';
import { AlertCircle, Activity, Terminal as TerminalIcon, TrendingUp, Shield, GitCompare, Network, Cpu, Brain, FileText, AlertTriangle } from 'lucide-react';
import * as Tabs from '@radix-ui/react-tabs';
import type { BatchedMetricPoint, CrashEvent } from './lib/api';
import './App.css';

function App() {
  const queryClient = useQueryClient();
  const [daemonHealthy, setDaemonHealthy] = useState(false);
  const [bootMarkers, setBootMarkers] = useState<Record<string, boolean>>({});
  const [currentMetricBatch, setCurrentMetricBatch] = useState<{
    points: BatchedMetricPoint[];
    seq?: number;
    droppedCount?: number;
  } | null>(null);
  const [currentCrashEvent, setCurrentCrashEvent] = useState<CrashEvent | null>(null);
  const [activeTab, setActiveTab] = useState('dashboard');
  const [explainDecision, setExplainDecision] = useState<AutonomyDecision | null>(null);

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
    }

    // Handle metric batch events
    if (event.type === 'metric_batch') {
      setCurrentMetricBatch({
        points: event.points,
        seq: event.seq,
        droppedCount: event.dropped_count,
      });
    }

    // Handle crash events
    if (event.type === 'crash') {
      setCurrentCrashEvent(event);
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
          <div className="h-full grid grid-cols-[320px,1fr] gap-4 p-4">
            {/* Left Sidebar - Controls & Terminal */}
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

              {/* Terminal */}
              <div className="bg-card rounded-lg border p-4 flex flex-col h-96">
                <div className="flex items-center gap-2 mb-4">
                  <TerminalIcon className="h-5 w-5" />
                  <h3 className="text-lg font-semibold">Terminal</h3>
                </div>
                <div className="flex-1 min-h-0">
                  <Terminal />
                </div>
              </div>
            </div>

            {/* Right Content - Tabbed Views */}
            <Tabs.Root value={activeTab} onValueChange={setActiveTab} className="flex flex-col h-full">
              <Tabs.List className="flex gap-2 border-b pb-2 mb-4 overflow-x-auto">
                <Tabs.Trigger
                  value="dashboard"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Activity className="h-4 w-4 inline-block mr-2" />
                  Dashboard
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="metrics"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <TrendingUp className="h-4 w-4 inline-block mr-2" />
                  Metrics
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="graph"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Network className="h-4 w-4 inline-block mr-2" />
                  Graph
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="scheduling"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Cpu className="h-4 w-4 inline-block mr-2" />
                  Scheduling
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="llm"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Brain className="h-4 w-4 inline-block mr-2" />
                  LLM
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="logs"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <FileText className="h-4 w-4 inline-block mr-2" />
                  Logs
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="crashes"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <AlertTriangle className="h-4 w-4 inline-block mr-2" />
                  Crashes
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="autonomy"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Shield className="h-4 w-4 inline-block mr-2" />
                  Autonomy
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="whatif"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <GitCompare className="h-4 w-4 inline-block mr-2" />
                  What-If
                </Tabs.Trigger>
                <Tabs.Trigger
                  value="memory"
                  className="px-4 py-2 rounded-md data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-muted transition-colors"
                >
                  <Shield className="h-4 w-4 inline-block mr-2" />
                  Memory
                </Tabs.Trigger>
              </Tabs.List>

              <Tabs.Content value="dashboard" className="flex-1 overflow-y-auto">
                <Dashboard daemonHealthy={daemonHealthy} />
              </Tabs.Content>

              <Tabs.Content value="metrics" className="flex-1 overflow-hidden">
                <MetricsPanel metricBatch={currentMetricBatch} />
              </Tabs.Content>

              <Tabs.Content value="graph" className="flex-1 overflow-hidden">
                <GraphPanel />
              </Tabs.Content>

              <Tabs.Content value="scheduling" className="flex-1 overflow-hidden">
                <SchedPanel />
              </Tabs.Content>

              <Tabs.Content value="llm" className="flex-1 overflow-hidden">
                <LlmPanel />
              </Tabs.Content>

              <Tabs.Content value="logs" className="flex-1 overflow-hidden">
                <LogsPanel />
              </Tabs.Content>

              <Tabs.Content value="crashes" className="flex-1 overflow-hidden">
                <CrashPanel crashEvent={currentCrashEvent} />
              </Tabs.Content>

              <Tabs.Content value="autonomy" className="flex-1 overflow-hidden">
                <AutonomyPanel onExplainDecision={setExplainDecision} />
              </Tabs.Content>

              <Tabs.Content value="whatif" className="flex-1 overflow-hidden">
                <WhatIfSimulator />
              </Tabs.Content>

              <Tabs.Content value="memory" className="flex-1 overflow-hidden">
                <ApprovalsPanel />
              </Tabs.Content>
            </Tabs.Root>
          </div>
        )}

        {/* ExplainView Modal */}
        {explainDecision && (
          <ExplainView
            decision={explainDecision}
            onClose={() => setExplainDecision(null)}
          />
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
