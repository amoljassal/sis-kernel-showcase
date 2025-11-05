/**
 * LLM Panel - Model loading, inference, and audit (feature-gated)
 */

import { useState, useRef, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { llmApi, type LoadModelRequest, type InferRequest, type AuditEntry, type LlmStatus } from '../lib/api';
import { Brain, Play, AlertCircle, Activity } from 'lucide-react';
import { useWebSocket, type LlmTokensEvent } from '../lib/useWebSocket';

export function LlmPanel() {
  const queryClient = useQueryClient();

  // State
  const [modelId, setModelId] = useState<string>('');
  const [wcetCycles, setWcetCycles] = useState<string>('');
  const [inferText, setInferText] = useState<string>('');
  const [maxTokens, setMaxTokens] = useState<string>('100');
  const [error, setError] = useState<string>('');

  // Token streaming state
  const [streamingOutput, setStreamingOutput] = useState<Record<string, string>>({});
  const [activeRequestId, setActiveRequestId] = useState<string | null>(null);

  // Virtualization ref
  const auditRef = useRef<HTMLDivElement>(null);

  // WebSocket subscription for llm_tokens events
  useWebSocket((event) => {
    if (event.type === 'llm_tokens') {
      const llmEvent = event as LlmTokensEvent;
      setStreamingOutput((prev) => ({
        ...prev,
        [llmEvent.requestId]: (prev[llmEvent.requestId] || '') + llmEvent.chunk,
      }));
      if (llmEvent.done) {
        // Move to active display
        setActiveRequestId(llmEvent.requestId);
        queryClient.invalidateQueries({ queryKey: ['llm', 'audit'] });
      }
    }
  });

  // Fetch LLM status
  const { data: status } = useQuery({
    queryKey: ['llm', 'status'],
    queryFn: () => llmApi.status(),
    refetchInterval: 3000,
    retry: 1,
  });

  // Fetch audit log
  const { data: auditLog = [] } = useQuery({
    queryKey: ['llm', 'audit'],
    queryFn: () => llmApi.audit(),
    refetchInterval: 5000,
    retry: 1,
  });

  // Load model mutation
  const loadModel = useMutation({
    mutationFn: (req: LoadModelRequest) => llmApi.load(req),
    onSuccess: () => {
      setError('');
      queryClient.invalidateQueries({ queryKey: ['llm', 'status'] });
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to load model');
    },
  });

  // Infer mutation
  const infer = useMutation({
    mutationFn: (req: InferRequest) => llmApi.infer(req),
    onSuccess: (data) => {
      setError('');
      setActiveRequestId(data.requestId);
      setStreamingOutput((prev) => ({ ...prev, [data.requestId]: '' }));
      setInferText('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to start inference');
    },
  });

  // Virtualized audit list
  const rowVirtualizer = useVirtualizer({
    count: auditLog.length,
    getScrollElement: () => auditRef.current,
    estimateSize: () => 48,
    overscan: 10,
  });

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-4 gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Brain className="h-6 w-6" />
          LLM Inference
        </h2>
        {status && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Activity className="h-4 w-4" />
            <span>Budget: {status.budget}</span>
            <span>Queue: {status.queueDepth}</span>
          </div>
        )}
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

      {/* Load Model Form */}
      <div className="bg-muted p-4 rounded-md space-y-3">
        <h3 className="font-semibold text-sm">Load Model</h3>
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">Model ID</label>
            <input
              type="text"
              value={modelId}
              onChange={(e) => setModelId(e.target.value)}
              placeholder="e.g., llama-7b"
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            />
          </div>
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">WCET Cycles (optional)</label>
            <input
              type="number"
              value={wcetCycles}
              onChange={(e) => setWcetCycles(e.target.value)}
              placeholder="e.g., 1000000"
              className="w-full px-3 py-2 bg-background border rounded-md text-sm"
            />
          </div>
        </div>
        <button
          onClick={() => {
            const req: LoadModelRequest = { modelId };
            if (wcetCycles) req.wcetCycles = parseInt(wcetCycles);
            loadModel.mutate(req);
          }}
          disabled={loadModel.isPending || !modelId}
          className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm"
        >
          {loadModel.isPending ? 'Loading...' : 'Load Model'}
        </button>
      </div>

      {/* Status Card */}
      {status && (
        <div className="bg-muted p-4 rounded-md">
          <h3 className="font-semibold text-sm mb-3">Model Status</h3>
          <div className="grid grid-cols-3 gap-4 text-sm">
            <div>
              <p className="text-muted-foreground text-xs">WCET Cycles</p>
              <p className="font-mono">{status.wcetCycles}</p>
            </div>
            <div>
              <p className="text-muted-foreground text-xs">Period (ns)</p>
              <p className="font-mono">{status.periodNs}</p>
            </div>
            <div>
              <p className="text-muted-foreground text-xs">Max Tokens/Period</p>
              <p className="font-mono">{status.maxTokensPerPeriod}</p>
            </div>
          </div>
        </div>
      )}

      {/* Inference Form */}
      <div className="bg-muted p-4 rounded-md space-y-3">
        <h3 className="font-semibold text-sm flex items-center gap-2">
          <Play className="h-4 w-4" />
          Run Inference
        </h3>
        <div className="space-y-2">
          <textarea
            value={inferText}
            onChange={(e) => setInferText(e.target.value)}
            placeholder="Enter prompt text..."
            className="w-full px-3 py-2 bg-background border rounded-md text-sm min-h-[100px] resize-y"
          />
          <div className="flex gap-2">
            <input
              type="number"
              value={maxTokens}
              onChange={(e) => setMaxTokens(e.target.value)}
              placeholder="Max tokens"
              className="w-32 px-3 py-2 bg-background border rounded-md text-sm"
            />
            <button
              onClick={() => {
                infer.mutate({ text: inferText, maxTokens: parseInt(maxTokens) });
              }}
              disabled={infer.isPending || !inferText}
              className="flex-1 px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 text-sm"
            >
              {infer.isPending ? 'Starting...' : 'Infer'}
            </button>
          </div>
        </div>
      </div>

      {/* Streaming Output */}
      {activeRequestId && streamingOutput[activeRequestId] !== undefined && (
        <div className="bg-muted p-4 rounded-md space-y-2">
          <h3 className="font-semibold text-sm">Output (Request: {activeRequestId.slice(0, 8)}...)</h3>
          <div className="bg-background p-3 rounded-md font-mono text-sm whitespace-pre-wrap max-h-[200px] overflow-y-auto">
            {streamingOutput[activeRequestId] || '(streaming...)'}
          </div>
        </div>
      )}

      {/* Audit Log */}
      <div className="flex-1 bg-muted rounded-md flex flex-col overflow-hidden">
        <div className="px-4 py-3 border-b border-border flex items-center justify-between">
          <h3 className="font-semibold">Audit Log</h3>
          <span className="text-sm text-muted-foreground">{auditLog.length} entries</span>
        </div>
        <div
          ref={auditRef}
          className="flex-1 overflow-y-auto"
          style={{ contain: 'strict' }}
        >
          {auditLog.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              No audit entries
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
                const entry = auditLog[virtualRow.index];

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
                        <span className="font-mono text-sm font-semibold w-24 truncate" title={entry.id}>
                          {entry.id.slice(0, 12)}...
                        </span>
                        <span className="text-sm text-muted-foreground min-w-[120px]">
                          {entry.modelId}
                        </span>
                      </div>
                      <div className="flex items-center gap-6 text-sm text-muted-foreground">
                        <span>{entry.tokens} tokens</span>
                        <span className={`px-2 py-1 rounded ${
                          entry.done ? 'bg-green-500/10 text-green-500' : 'bg-yellow-500/10 text-yellow-500'
                        }`}>
                          {entry.done ? 'Done' : 'Running'}
                        </span>
                        <span className="font-mono text-xs">
                          {new Date(entry.ts).toLocaleTimeString()}
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
