/**
 * Crash Panel - Live crash feed with incident workflow
 */

import { useState, useEffect, useMemo } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import {
  AlertTriangle,
  FileText,
  X,
  PlusCircle,
  Filter,
  Eye,
} from 'lucide-react';
import {
  crashApi,
  incidentApi,
  type CrashLog,
  type CrashEvent,
  type Incident,
} from '../lib/api';
import { showSuccessToast, showErrorToast } from '../lib/toast';

interface CrashPanelProps {
  /** Incoming crash event from WebSocket */
  crashEvent?: CrashEvent | null;
}

type SeverityFilter = 'all' | 'critical' | 'high' | 'medium' | 'low';

export function CrashPanel({ crashEvent }: CrashPanelProps) {
  const [crashes, setCrashes] = useState<CrashLog[]>([]);
  const [selectedCrash, setSelectedCrash] = useState<CrashLog | null>(null);
  const [showIncidentDialog, setShowIncidentDialog] = useState(false);
  const [incidentTitle, setIncidentTitle] = useState('');
  const [incidentDescription, setIncidentDescription] = useState('');
  const [severityFilter, setSeverityFilter] = useState<SeverityFilter>('all');

  // Fetch crash list
  const { data: crashListData, refetch: refetchCrashes } = useQuery({
    queryKey: ['crashes', severityFilter],
    queryFn: () =>
      crashApi.list({
        page: 1,
        page_size: 100,
        severity: severityFilter !== 'all' ? severityFilter : undefined,
      }),
    refetchInterval: 5000,
  });

  // Fetch incidents
  const { data: incidentListData } = useQuery({
    queryKey: ['incidents'],
    queryFn: () => incidentApi.list({ page: 1, page_size: 100 }),
    refetchInterval: 5000,
  });

  // Create incident mutation
  const createIncidentMutation = useMutation({
    mutationFn: incidentApi.create,
    onSuccess: () => {
      showSuccessToast('Incident created successfully');
      setShowIncidentDialog(false);
      setIncidentTitle('');
      setIncidentDescription('');
    },
    onError: (err: any) => {
      showErrorToast(err.message || 'Failed to create incident');
    },
  });

  // Handle incoming crash event from WebSocket
  useEffect(() => {
    if (!crashEvent) return;

    const newCrash: CrashLog = {
      crashId: crashEvent.crashId,
      ts: crashEvent.ts,
      panic_msg: crashEvent.panicMsg,
      stack_trace: crashEvent.stackTrace,
      severity: crashEvent.severity,
    };

    setCrashes((prev) => {
      // Deduplicate by crashId
      const existing = prev.find((c) => c.crashId === crashEvent.crashId);
      if (existing) return prev;
      return [newCrash, ...prev].slice(0, 100); // Keep last 100
    });

    // Refetch to sync with backend
    refetchCrashes();
  }, [crashEvent, refetchCrashes]);

  // Merge WebSocket crashes with REST crashes
  const allCrashes = useMemo(() => {
    const restCrashes = crashListData?.crashes || [];
    const merged = [...crashes];

    // Add REST crashes that aren't in WebSocket list
    for (const restCrash of restCrashes) {
      if (!merged.find((c) => c.crashId === restCrash.crashId)) {
        merged.push(restCrash);
      }
    }

    // Sort by timestamp desc
    return merged.sort((a, b) => b.ts - a.ts);
  }, [crashes, crashListData]);

  const handleCreateIncident = () => {
    if (!selectedCrash) return;

    createIncidentMutation.mutate({
      crashId: selectedCrash.crashId,
      title: incidentTitle,
      description: incidentDescription,
    });
  };

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'text-red-500 bg-red-500/10 border-red-500/20';
      case 'high':
        return 'text-orange-500 bg-orange-500/10 border-orange-500/20';
      case 'medium':
        return 'text-yellow-500 bg-yellow-500/10 border-yellow-500/20';
      case 'low':
        return 'text-blue-500 bg-blue-500/10 border-blue-500/20';
      default:
        return 'text-muted-foreground bg-muted/10 border-muted/20';
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="p-4 border-b space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            Crash Capture
          </h3>
          <div className="text-xs text-muted-foreground">
            {allCrashes.length} crash{allCrashes.length !== 1 ? 'es' : ''}
          </div>
        </div>

        {/* Severity Filter */}
        <div className="flex items-center gap-2">
          <Filter className="h-4 w-4 text-muted-foreground" />
          <div className="flex gap-1 flex-wrap">
            {(['all', 'critical', 'high', 'medium', 'low'] as SeverityFilter[]).map(
              (severity) => (
                <button
                  key={severity}
                  onClick={() => setSeverityFilter(severity)}
                  className={`px-2 py-1 rounded-md text-xs font-medium transition-colors ${
                    severityFilter === severity
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80'
                  }`}
                >
                  {severity}
                </button>
              )
            )}
          </div>
        </div>
      </div>

      {/* Crash List */}
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {allCrashes.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
            No crashes captured
          </div>
        ) : (
          allCrashes.map((crash) => {
            const crashDate = new Date(crash.ts);
            const linkedIncident = incidentListData?.incidents.find(
              (i) => i.crashId === crash.crashId
            );

            return (
              <div
                key={crash.crashId}
                className={`p-3 rounded-md border transition-colors cursor-pointer ${
                  selectedCrash?.crashId === crash.crashId
                    ? 'bg-primary/10 border-primary'
                    : 'hover:bg-muted/50'
                }`}
                onClick={() => setSelectedCrash(crash)}
              >
                <div className="flex items-start gap-3">
                  {/* Severity badge */}
                  <div
                    className={`px-2 py-0.5 rounded text-xs font-semibold border ${getSeverityColor(crash.severity)}`}
                  >
                    {crash.severity.toUpperCase()}
                  </div>

                  {/* Crash info */}
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">
                      {crash.panic_msg}
                    </p>
                    <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                      <span>{crashDate.toLocaleString()}</span>
                      {crash.run_id && <span>Run: {crash.run_id.slice(0, 8)}</span>}
                      {linkedIncident && (
                        <span className="text-blue-500">
                          Incident: {linkedIncident.incidentId.slice(0, 8)}
                        </span>
                      )}
                    </div>
                  </div>

                  {/* Actions */}
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedCrash(crash);
                    }}
                    className="flex-shrink-0 p-1 hover:bg-accent rounded"
                    title="View details"
                  >
                    <Eye className="h-4 w-4" />
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* Selected Crash Detail Modal */}
      {selectedCrash && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card rounded-lg border shadow-lg max-w-3xl w-full max-h-[80vh] flex flex-col">
            {/* Modal Header */}
            <div className="p-4 border-b flex items-center justify-between">
              <h4 className="font-semibold">Crash Details</h4>
              <button
                onClick={() => setSelectedCrash(null)}
                className="p-1 hover:bg-accent rounded"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            {/* Modal Content */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              <div>
                <label className="text-xs font-semibold text-muted-foreground uppercase">
                  Crash ID
                </label>
                <p className="text-sm font-mono">{selectedCrash.crashId}</p>
              </div>

              <div>
                <label className="text-xs font-semibold text-muted-foreground uppercase">
                  Panic Message
                </label>
                <p className="text-sm">{selectedCrash.panic_msg}</p>
              </div>

              <div>
                <label className="text-xs font-semibold text-muted-foreground uppercase">
                  Severity
                </label>
                <div
                  className={`inline-block px-2 py-0.5 rounded text-xs font-semibold border ${getSeverityColor(selectedCrash.severity)}`}
                >
                  {selectedCrash.severity.toUpperCase()}
                </div>
              </div>

              <div>
                <label className="text-xs font-semibold text-muted-foreground uppercase">
                  Timestamp
                </label>
                <p className="text-sm">{new Date(selectedCrash.ts).toLocaleString()}</p>
              </div>

              {selectedCrash.stack_trace && selectedCrash.stack_trace.length > 0 && (
                <div>
                  <label className="text-xs font-semibold text-muted-foreground uppercase">
                    Stack Trace
                  </label>
                  <div className="mt-1 p-3 bg-muted rounded-md font-mono text-xs overflow-x-auto">
                    {selectedCrash.stack_trace.map((frame, index) => (
                      <div key={index}>{frame}</div>
                    ))}
                  </div>
                </div>
              )}

              {selectedCrash.registers && (
                <div>
                  <label className="text-xs font-semibold text-muted-foreground uppercase">
                    Registers
                  </label>
                  <pre className="mt-1 p-3 bg-muted rounded-md font-mono text-xs overflow-x-auto">
                    {JSON.stringify(selectedCrash.registers, null, 2)}
                  </pre>
                </div>
              )}
            </div>

            {/* Modal Footer */}
            <div className="p-4 border-t flex justify-end gap-2">
              {!showIncidentDialog ? (
                <button
                  onClick={() => setShowIncidentDialog(true)}
                  className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90"
                >
                  <PlusCircle className="h-4 w-4" />
                  Create Incident
                </button>
              ) : (
                <div className="flex-1 space-y-3">
                  <input
                    type="text"
                    value={incidentTitle}
                    onChange={(e) => setIncidentTitle(e.target.value)}
                    placeholder="Incident title"
                    className="w-full px-3 py-2 text-sm border rounded-md bg-background"
                    autoFocus
                  />
                  <textarea
                    value={incidentDescription}
                    onChange={(e) => setIncidentDescription(e.target.value)}
                    placeholder="Description"
                    className="w-full px-3 py-2 text-sm border rounded-md bg-background resize-none"
                    rows={3}
                  />
                  <div className="flex gap-2">
                    <button
                      onClick={handleCreateIncident}
                      disabled={!incidentTitle.trim() || createIncidentMutation.isPending}
                      className="flex-1 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                    >
                      {createIncidentMutation.isPending ? 'Creating...' : 'Create'}
                    </button>
                    <button
                      onClick={() => {
                        setShowIncidentDialog(false);
                        setIncidentTitle('');
                        setIncidentDescription('');
                      }}
                      className="px-4 py-2 bg-muted hover:bg-muted/80 rounded-md"
                    >
                      Cancel
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
