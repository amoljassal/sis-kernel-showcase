/**
 * Graph Panel - Graph control and operator management
 */

import { useState, useRef, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import { graphApi, type GraphState } from '../lib/api';
import { type GraphStateEvent } from '../lib/useWebSocket';
import { Plus, Play, TrendingUp, MessageSquare, Download, AlertTriangle } from 'lucide-react';

interface GraphPanelProps {
  onGraphStateEvent?: (event: GraphStateEvent) => void;
}

export function GraphPanel({ onGraphStateEvent }: GraphPanelProps) {
  const queryClient = useQueryClient();

  // State
  const [graphId, setGraphId] = useState<string>('');
  const [channelCap, setChannelCap] = useState<string>('100');
  const [operatorId, setOperatorId] = useState<string>('');
  const [operatorPrio, setOperatorPrio] = useState<string>('100');
  const [operatorStage, setOperatorStage] = useState<string>('');
  const [steps, setSteps] = useState<string>('10');
  const [predictOpId, setPredictOpId] = useState<string>('');
  const [predictLatency, setPredictLatency] = useState<string>('1000');
  const [predictDepth, setPredictDepth] = useState<string>('5');
  const [feedbackOpId, setFeedbackOpId] = useState<string>('');
  const [feedbackVerdict, setFeedbackVerdict] = useState<'helpful' | 'not_helpful' | 'expected'>('helpful');
  const [predictionResult, setPredictionResult] = useState<string>('');
  const [error, setError] = useState<string>('');

  // Virtualization refs
  const operatorsRef = useRef<HTMLDivElement>(null);
  const channelsRef = useRef<HTMLDivElement>(null);

  // Subscribe to graph state via WS
  useEffect(() => {
    if (onGraphStateEvent) {
      // This would be called from App.tsx when WS event arrives
      // For now, we'll poll via REST
    }
  }, [onGraphStateEvent]);

  // Fetch graph state
  const { data: graphState, refetch: refetchState } = useQuery({
    queryKey: ['graph', 'state', graphId],
    queryFn: () => graphApi.state(graphId),
    enabled: !!graphId,
    refetchInterval: 2000,
    retry: 1,
  });

  // Create graph mutation
  const createGraph = useMutation({
    mutationFn: () => graphApi.create(),
    onSuccess: (data) => {
      setGraphId(data.graphId);
      setError('');
      queryClient.invalidateQueries({ queryKey: ['graph'] });
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to create graph');
    },
  });

  // Add channel mutation
  const addChannel = useMutation({
    mutationFn: () => graphApi.addChannel({
      graphId,
      cap: parseInt(channelCap),
    }),
    onSuccess: () => {
      setError('');
      refetchState();
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to add channel');
    },
  });

  // Add operator mutation
  const addOperator = useMutation({
    mutationFn: () => graphApi.addOperator({
      graphId,
      opId: operatorId,
      prio: operatorPrio ? parseInt(operatorPrio) : undefined,
      stage: operatorStage || undefined,
    }),
    onSuccess: () => {
      setError('');
      setOperatorId('');
      refetchState();
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to add operator');
    },
  });

  // Start graph mutation
  const startGraph = useMutation({
    mutationFn: () => graphApi.start({
      graphId,
      steps: parseInt(steps),
    }),
    onSuccess: () => {
      setError('');
      refetchState();
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to start graph');
    },
  });

  // Predict mutation
  const predict = useMutation({
    mutationFn: () => graphApi.predict({
      opId: predictOpId,
      latency_us: parseInt(predictLatency),
      depth: parseInt(predictDepth),
    }),
    onSuccess: (data) => {
      setPredictionResult(`Predicted: ${data.predicted.toFixed(2)}${data.conf ? ` (conf: ${data.conf.toFixed(2)})` : ''}`);
      setError('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to predict');
    },
  });

  // Feedback mutation
  const submitFeedback = useMutation({
    mutationFn: () => graphApi.feedback({
      opId: feedbackOpId,
      verdict: feedbackVerdict,
    }),
    onSuccess: () => {
      setFeedbackOpId('');
      setError('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to submit feedback');
    },
  });

  // Export mutation
  const exportGraph = useMutation({
    mutationFn: () => graphApi.export({
      graphId,
      format: 'json',
    }),
    onSuccess: (data) => {
      const blob = new Blob([data.json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `graph-${graphId}.json`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      setError('');
    },
    onError: (err: any) => {
      setError(err.response?.data?.detail || 'Failed to export graph');
    },
  });

  // Virtualized operators
  const operators = graphState?.state?.operators || [];
  const operatorVirtualizer = useVirtualizer({
    count: operators.length,
    getScrollElement: () => operatorsRef.current,
    estimateSize: () => 48,
    overscan: 5,
  });

  // Virtualized channels
  const channels = graphState?.state?.channels || [];
  const channelVirtualizer = useVirtualizer({
    count: channels.length,
    getScrollElement: () => channelsRef.current,
    estimateSize: () => 48,
    overscan: 5,
  });

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-4 gap-4 overflow-y-auto">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold">Graph Control</h2>
        {graphId && (
          <span className="text-sm text-muted-foreground">Graph ID: {graphId}</span>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-destructive/10 text-destructive px-4 py-3 rounded-md flex items-start gap-2">
          <AlertTriangle className="h-5 w-5 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-semibold">Error</p>
            <p className="text-sm">{error}</p>
          </div>
          <button onClick={() => setError('')} className="text-sm hover:underline">
            Dismiss
          </button>
        </div>
      )}

      {/* Create Graph */}
      {!graphId && (
        <div className="bg-muted p-4 rounded-md">
          <button
            onClick={() => createGraph.mutate()}
            disabled={createGraph.isPending}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
          >
            {createGraph.isPending ? 'Creating...' : 'Create Graph'}
          </button>
        </div>
      )}

      {graphId && (
        <>
          {/* Controls Grid */}
          <div className="grid grid-cols-2 gap-4">
            {/* Add Channel */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <Plus className="h-4 w-4" />
                Add Channel
              </h3>
              <div className="space-y-2">
                <input
                  type="number"
                  value={channelCap}
                  onChange={(e) => setChannelCap(e.target.value)}
                  placeholder="Capacity"
                  className="w-full px-3 py-2 bg-background border rounded-md"
                />
                <button
                  onClick={() => addChannel.mutate()}
                  disabled={addChannel.isPending}
                  className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                >
                  {addChannel.isPending ? 'Adding...' : 'Add Channel'}
                </button>
              </div>
            </div>

            {/* Add Operator */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <Plus className="h-4 w-4" />
                Add Operator
              </h3>
              <div className="space-y-2">
                <input
                  type="text"
                  value={operatorId}
                  onChange={(e) => setOperatorId(e.target.value)}
                  placeholder="Operator ID"
                  className="w-full px-3 py-2 bg-background border rounded-md"
                />
                <div className="grid grid-cols-2 gap-2">
                  <input
                    type="number"
                    value={operatorPrio}
                    onChange={(e) => setOperatorPrio(e.target.value)}
                    placeholder="Priority"
                    className="px-3 py-2 bg-background border rounded-md"
                  />
                  <input
                    type="text"
                    value={operatorStage}
                    onChange={(e) => setOperatorStage(e.target.value)}
                    placeholder="Stage"
                    className="px-3 py-2 bg-background border rounded-md"
                  />
                </div>
                <button
                  onClick={() => addOperator.mutate()}
                  disabled={addOperator.isPending || !operatorId}
                  className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                >
                  {addOperator.isPending ? 'Adding...' : 'Add Operator'}
                </button>
              </div>
            </div>

            {/* Start Graph */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <Play className="h-4 w-4" />
                Start Graph
              </h3>
              <div className="space-y-2">
                <input
                  type="number"
                  value={steps}
                  onChange={(e) => setSteps(e.target.value)}
                  placeholder="Steps"
                  className="w-full px-3 py-2 bg-background border rounded-md"
                />
                <button
                  onClick={() => startGraph.mutate()}
                  disabled={startGraph.isPending}
                  className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                >
                  {startGraph.isPending ? 'Starting...' : 'Start'}
                </button>
              </div>
            </div>

            {/* Predict */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <TrendingUp className="h-4 w-4" />
                Predict
              </h3>
              <div className="space-y-2">
                <input
                  type="text"
                  value={predictOpId}
                  onChange={(e) => setPredictOpId(e.target.value)}
                  placeholder="Operator ID"
                  className="w-full px-3 py-2 bg-background border rounded-md"
                />
                <div className="grid grid-cols-2 gap-2">
                  <input
                    type="number"
                    value={predictLatency}
                    onChange={(e) => setPredictLatency(e.target.value)}
                    placeholder="Latency (μs)"
                    className="px-3 py-2 bg-background border rounded-md"
                  />
                  <input
                    type="number"
                    value={predictDepth}
                    onChange={(e) => setPredictDepth(e.target.value)}
                    placeholder="Depth"
                    className="px-3 py-2 bg-background border rounded-md"
                  />
                </div>
                <button
                  onClick={() => predict.mutate()}
                  disabled={predict.isPending || !predictOpId}
                  className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                >
                  {predict.isPending ? 'Predicting...' : 'Predict'}
                </button>
                {predictionResult && (
                  <p className="text-sm text-green-500">{predictionResult}</p>
                )}
              </div>
            </div>
          </div>

          {/* Feedback & Export */}
          <div className="grid grid-cols-2 gap-4">
            {/* Feedback */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <MessageSquare className="h-4 w-4" />
                Feedback
              </h3>
              <div className="space-y-2">
                <input
                  type="text"
                  value={feedbackOpId}
                  onChange={(e) => setFeedbackOpId(e.target.value)}
                  placeholder="Operator ID"
                  className="w-full px-3 py-2 bg-background border rounded-md"
                />
                <select
                  value={feedbackVerdict}
                  onChange={(e) => setFeedbackVerdict(e.target.value as any)}
                  className="w-full px-3 py-2 bg-background border rounded-md"
                >
                  <option value="helpful">Helpful</option>
                  <option value="not_helpful">Not Helpful</option>
                  <option value="expected">Expected</option>
                </select>
                <button
                  onClick={() => submitFeedback.mutate()}
                  disabled={submitFeedback.isPending || !feedbackOpId}
                  className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
                >
                  {submitFeedback.isPending ? 'Submitting...' : 'Submit Feedback'}
                </button>
              </div>
            </div>

            {/* Export */}
            <div className="bg-muted p-4 rounded-md space-y-2">
              <h3 className="font-semibold flex items-center gap-2">
                <Download className="h-4 w-4" />
                Export
              </h3>
              <button
                onClick={() => exportGraph.mutate()}
                disabled={exportGraph.isPending}
                className="w-full px-3 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
              >
                {exportGraph.isPending ? 'Exporting...' : 'Export JSON'}
              </button>
            </div>
          </div>

          {/* State View */}
          <div className="grid grid-cols-2 gap-4 flex-1 min-h-0">
            {/* Operators Table */}
            <div className="bg-muted rounded-md flex flex-col">
              <div className="px-4 py-3 border-b border-border">
                <h3 className="font-semibold">Operators ({operators.length})</h3>
              </div>
              <div
                ref={operatorsRef}
                className="flex-1 overflow-y-auto"
                style={{ contain: 'strict' }}
              >
                <div
                  style={{
                    height: `${operatorVirtualizer.getTotalSize()}px`,
                    width: '100%',
                    position: 'relative',
                  }}
                >
                  {operatorVirtualizer.getVirtualItems().map((virtualRow) => {
                    const op = operators[virtualRow.index];
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
                        className="px-4 py-2 border-b border-border hover:bg-muted-foreground/5"
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex flex-col">
                            <span className="font-mono text-sm">{op.id}</span>
                            {op.stage && (
                              <span className="text-xs text-muted-foreground">{op.stage}</span>
                            )}
                          </div>
                          <div className="flex gap-4 text-sm">
                            {op.prio !== undefined && (
                              <span className="text-muted-foreground">Prio: {op.prio}</span>
                            )}
                            {op.stats && (
                              <span className="text-green-500">
                                {op.stats.execCount} runs, {op.stats.avgUs.toFixed(1)}μs avg
                              </span>
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>

            {/* Channels Table */}
            <div className="bg-muted rounded-md flex flex-col">
              <div className="px-4 py-3 border-b border-border">
                <h3 className="font-semibold">Channels ({channels.length})</h3>
              </div>
              <div
                ref={channelsRef}
                className="flex-1 overflow-y-auto"
                style={{ contain: 'strict' }}
              >
                <div
                  style={{
                    height: `${channelVirtualizer.getTotalSize()}px`,
                    width: '100%',
                    position: 'relative',
                  }}
                >
                  {channelVirtualizer.getVirtualItems().map((virtualRow) => {
                    const ch = channels[virtualRow.index];
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
                        className="px-4 py-2 border-b border-border hover:bg-muted-foreground/5"
                      >
                        <div className="flex items-center justify-between">
                          <span className="font-mono text-sm">{ch.id}</span>
                          <div className="flex gap-4 text-sm text-muted-foreground">
                            <span>Cap: {ch.cap}</span>
                            {ch.depth !== undefined && (
                              <span className="text-blue-500">Depth: {ch.depth}</span>
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>
        </>
      )}
    </div>
  );
}
