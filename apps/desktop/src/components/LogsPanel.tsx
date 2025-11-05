/**
 * Logs Panel - Log viewer, run history, and self-check integration
 */

import { useState, useRef, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { logsApi, type LogEntry, type RunHistoryEntry, type StartRunRequest } from '../lib/api';
import { FileText, Play, Square, Download, CheckCircle2, Filter, Copy } from 'lucide-react';
import { useWebSocket, type LogLineEvent } from '../lib/useWebSocket';
import { copyJSONToClipboard } from '../lib/clipboard';

const MAX_LOG_BUFFER = 10000; // Ring buffer size

export function LogsPanel() {
  const queryClient = useQueryClient();

  // State
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [levelFilter, setLevelFilter] = useState<string>('');
  const [sourceFilter, setSourceFilter] = useState<string>('');
  const [searchText, setSearchText] = useState<string>('');
  const [runNote, setRunNote] = useState<string>('');
  const [selfCheckPassed, setSelfCheckPassed] = useState<boolean>(false);

  // Virtualization refs
  const logsRef = useRef<HTMLDivElement>(null);
  const runsRef = useRef<HTMLDivElement>(null);

  // WebSocket subscription for log_line events
  useWebSocket((event) => {
    if (event.type === 'log_line') {
      const logEvent = event as LogLineEvent;
      setLogs((prev) => {
        const newLogs = [
          ...prev,
          {
            ts: logEvent.ts,
            level: logEvent.level,
            source: logEvent.source,
            msg: logEvent.msg,
          },
        ];
        // Ring buffer: keep only last MAX_LOG_BUFFER entries
        if (newLogs.length > MAX_LOG_BUFFER) {
          return newLogs.slice(newLogs.length - MAX_LOG_BUFFER);
        }
        return newLogs;
      });
    }

    // Self-check completion detection
    if (event.type === 'self_check_completed') {
      setSelfCheckPassed((event as any).success);
    }
  });

  // Fetch run history
  const { data: runHistory = [] } = useQuery({
    queryKey: ['runs', 'list'],
    queryFn: () => logsApi.listRuns(),
    refetchInterval: 5000,
    retry: 1,
  });

  // Start run mutation
  const startRun = useMutation({
    mutationFn: (req: StartRunRequest) => logsApi.startRun(req),
    onSuccess: () => {
      setRunNote('');
      queryClient.invalidateQueries({ queryKey: ['runs', 'list'] });
    },
  });

  // Stop run mutation
  const stopRun = useMutation({
    mutationFn: () => logsApi.stopRun(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['runs', 'list'] });
    },
  });

  // Export run mutation
  const exportRun = useMutation({
    mutationFn: (runId: string) => logsApi.exportRun(runId),
    onSuccess: (data, runId) => {
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `run-${runId}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    },
  });

  // Filter logs
  const filteredLogs = logs.filter((log) => {
    if (levelFilter && log.level !== levelFilter) return false;
    if (sourceFilter && log.source !== sourceFilter) return false;
    if (searchText && !log.msg.toLowerCase().includes(searchText.toLowerCase())) return false;
    return true;
  });

  // Virtualized logs
  const logsVirtualizer = useVirtualizer({
    count: filteredLogs.length,
    getScrollElement: () => logsRef.current,
    estimateSize: () => 32,
    overscan: 20,
  });

  // Virtualized run history
  const runsVirtualizer = useVirtualizer({
    count: runHistory.length,
    getScrollElement: () => runsRef.current,
    estimateSize: () => 56,
    overscan: 10,
  });

  // Export logs as JSON
  const exportLogsJSON = () => {
    const blob = new Blob([JSON.stringify(filteredLogs, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logs-${Date.now()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Export logs as CSV
  const exportLogsCSV = () => {
    const csv = [
      'timestamp,level,source,message',
      ...filteredLogs.map((log) =>
        `${log.ts},"${log.level}","${log.source}","${log.msg.replace(/"/g, '""')}"`
      ),
    ].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `logs-${Date.now()}.csv`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  // Level color helper
  const getLevelColor = (level: string) => {
    switch (level) {
      case 'error':
        return 'text-red-500';
      case 'warn':
        return 'text-yellow-500';
      case 'info':
        return 'text-blue-500';
      case 'debug':
        return 'text-muted-foreground';
      default:
        return '';
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-4 gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <FileText className="h-6 w-6" />
          Logs & Runs
        </h2>
        <div className="flex gap-2">
          <button
            onClick={exportLogsJSON}
            className="px-3 py-1.5 bg-muted hover:bg-muted-foreground/10 rounded-md text-sm flex items-center gap-2"
          >
            <Download className="h-4 w-4" />
            JSON
          </button>
          <button
            onClick={exportLogsCSV}
            className="px-3 py-1.5 bg-muted hover:bg-muted-foreground/10 rounded-md text-sm flex items-center gap-2"
          >
            <Download className="h-4 w-4" />
            CSV
          </button>
          <button
            onClick={async () => {
              await copyJSONToClipboard(filteredLogs, 'Logs copied to clipboard');
            }}
            className="px-3 py-1.5 bg-muted hover:bg-muted-foreground/10 rounded-md text-sm flex items-center gap-2"
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                e.currentTarget.click();
              }
            }}
            tabIndex={0}
          >
            <Copy className="h-4 w-4" />
            Copy
          </button>
        </div>
      </div>

      {/* Self-Check PASS Banner */}
      {selfCheckPassed && (
        <div className="bg-green-500/10 text-green-500 px-4 py-3 rounded-md flex items-center gap-2 font-semibold">
          <CheckCircle2 className="h-5 w-5" />
          ALL MARKERS SEEN - SELF-CHECK PASSED
        </div>
      )}

      {/* Run Controls */}
      <div className="bg-muted p-4 rounded-md space-y-3">
        <h3 className="font-semibold text-sm">Run Control</h3>
        <div className="flex gap-2">
          <input
            type="text"
            value={runNote}
            onChange={(e) => setRunNote(e.target.value)}
            placeholder="Run note (optional)"
            className="flex-1 px-3 py-2 bg-background border rounded-md text-sm"
          />
          <button
            onClick={() => {
              startRun.mutate({
                profile: { features: [], bringup: true },
                note: runNote || undefined,
              });
            }}
            disabled={startRun.isPending}
            className="px-4 py-2 bg-green-500 text-white rounded-md hover:bg-green-600 disabled:opacity-50 text-sm flex items-center gap-2"
          >
            <Play className="h-4 w-4" />
            {startRun.isPending ? 'Starting...' : 'Start Run'}
          </button>
          <button
            onClick={() => stopRun.mutate()}
            disabled={stopRun.isPending}
            className="px-4 py-2 bg-red-500 text-white rounded-md hover:bg-red-600 disabled:opacity-50 text-sm flex items-center gap-2"
          >
            <Square className="h-4 w-4" />
            {stopRun.isPending ? 'Stopping...' : 'Stop Run'}
          </button>
        </div>
      </div>

      {/* Log Filters */}
      <div className="bg-muted p-4 rounded-md space-y-3">
        <h3 className="font-semibold text-sm flex items-center gap-2">
          <Filter className="h-4 w-4" />
          Filters
        </h3>
        <div className="grid grid-cols-3 gap-3">
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Level</label>
            <select
              value={levelFilter}
              onChange={(e) => setLevelFilter(e.target.value)}
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            >
              <option value="">All</option>
              <option value="debug">Debug</option>
              <option value="info">Info</option>
              <option value="warn">Warn</option>
              <option value="error">Error</option>
            </select>
          </div>
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Source</label>
            <select
              value={sourceFilter}
              onChange={(e) => setSourceFilter(e.target.value)}
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            >
              <option value="">All</option>
              <option value="daemon">Daemon</option>
              <option value="qemu">QEMU</option>
              <option value="kernel">Kernel</option>
            </select>
          </div>
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Search</label>
            <input
              type="text"
              value={searchText}
              onChange={(e) => setSearchText(e.target.value)}
              placeholder="Filter messages..."
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            />
          </div>
        </div>
        <div className="text-xs text-muted-foreground">
          Showing {filteredLogs.length} of {logs.length} logs (buffer: {MAX_LOG_BUFFER})
        </div>
      </div>

      {/* Log Viewer */}
      <div className="flex-1 bg-muted rounded-md flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b border-border">
          <h3 className="font-semibold">Live Logs</h3>
        </div>
        <div
          ref={logsRef}
          className="flex-1 overflow-y-auto font-mono text-xs"
          style={{ contain: 'strict' }}
        >
          {filteredLogs.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              No logs available
            </div>
          ) : (
            <div
              style={{
                height: `${logsVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {logsVirtualizer.getVirtualItems().map((virtualRow) => {
                const log = filteredLogs[virtualRow.index];

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
                    className="px-4 py-1 hover:bg-muted-foreground/5 border-b border-border/50"
                  >
                    <div className="flex items-start gap-2">
                      <span className="text-muted-foreground text-[10px] w-16 flex-shrink-0">
                        {new Date(log.ts).toLocaleTimeString()}
                      </span>
                      <span className={`w-12 flex-shrink-0 uppercase font-semibold ${getLevelColor(log.level)}`}>
                        {log.level}
                      </span>
                      <span className="w-16 flex-shrink-0 text-muted-foreground">
                        [{log.source}]
                      </span>
                      <span className="flex-1 break-all">{log.msg}</span>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Run History */}
      <div className="h-48 bg-muted rounded-md flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b border-border flex items-center justify-between">
          <h3 className="font-semibold">Run History</h3>
          <span className="text-sm text-muted-foreground">{runHistory.length} runs</span>
        </div>
        <div
          ref={runsRef}
          className="flex-1 overflow-y-auto"
          style={{ contain: 'strict' }}
        >
          {runHistory.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              No run history
            </div>
          ) : (
            <div
              style={{
                height: `${runsVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {runsVirtualizer.getVirtualItems().map((virtualRow) => {
                const run = runHistory[virtualRow.index];

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
                    className="px-4 py-3 border-b border-border"
                  >
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-4">
                        <span className="font-mono text-sm font-semibold w-24 truncate" title={run.runId}>
                          {run.runId.slice(0, 12)}...
                        </span>
                        <span className="text-sm text-muted-foreground">
                          {new Date(run.startedAt).toLocaleString()}
                        </span>
                        {run.stoppedAt && (
                          <span className="text-xs text-muted-foreground">
                            → {new Date(run.stoppedAt).toLocaleString()}
                          </span>
                        )}
                      </div>
                      <button
                        onClick={() => exportRun.mutate(run.runId)}
                        disabled={exportRun.isPending}
                        className="px-3 py-1 bg-primary/10 hover:bg-primary/20 text-primary rounded-md text-xs flex items-center gap-1"
                      >
                        <Download className="h-3 w-3" />
                        Export
                      </button>
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
