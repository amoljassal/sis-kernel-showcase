/**
 * Model drift detection panel with accuracy monitoring and retrain controls
 */

import { useState, useRef, useMemo } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  driftApi,
  DriftStatus,
  DriftSample,
  RetrainRequest,
} from '../lib/api';
import {
  TrendingDown,
  AlertTriangle,
  CheckCircle,
  Activity,
  RotateCcw,
  Play,
  Target,
  Clock,
} from 'lucide-react';

export function DriftPanel() {
  const queryClient = useQueryClient();
  const tableRef = useRef<HTMLDivElement>(null);
  const [trainingExamples, setTrainingExamples] = useState('1000');
  const [epochs, setEpochs] = useState('10');

  // Fetch drift status
  const { data: status, error: statusError } = useQuery({
    queryKey: ['drift', 'status'],
    queryFn: () => driftApi.getStatus(),
    refetchInterval: 2000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  // Fetch drift history
  const { data: historyData } = useQuery({
    queryKey: ['drift', 'history'],
    queryFn: () => driftApi.getHistory({ limit: 200 }),
    refetchInterval: 5000,
  });

  const samples = historyData?.samples ?? [];

  // Virtualizer for samples table
  const rowVirtualizer = useVirtualizer({
    count: samples.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 56,
    overscan: 10,
  });

  // Mutations
  const retrainMutation = useMutation({
    mutationFn: (req: RetrainRequest) => driftApi.triggerRetrain(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['drift'] });
    },
  });

  const resetBaselineMutation = useMutation({
    mutationFn: () => driftApi.resetBaseline(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['drift'] });
    },
  });

  // Helper to get drift level badge
  const getDriftLevelBadge = (level: DriftSample['drift_level']) => {
    switch (level) {
      case 'normal':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">
            <CheckCircle className="h-3 w-3" />
            Normal
          </span>
        );
      case 'warning':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full">
            <AlertTriangle className="h-3 w-3" />
            Warning
          </span>
        );
      case 'critical':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <TrendingDown className="h-3 w-3" />
            Critical
          </span>
        );
    }
  };

  // Calculate chart data for accuracy trend
  const chartData = useMemo(() => {
    if (samples.length === 0) return null;

    const maxSamples = 50;
    const recentSamples = samples.slice(0, maxSamples).reverse();
    const maxAccuracy = Math.max(...recentSamples.map(s => s.accuracy), status?.baseline_accuracy ?? 0);
    const minAccuracy = Math.min(...recentSamples.map(s => s.accuracy));
    const range = maxAccuracy - minAccuracy || 0.1;

    return {
      samples: recentSamples,
      maxAccuracy,
      minAccuracy,
      range,
    };
  }, [samples, status]);

  // Handle retrain
  const handleRetrain = () => {
    const examples = parseInt(trainingExamples, 10);
    const ep = parseInt(epochs, 10);
    if (!isNaN(examples) && !isNaN(ep) && examples > 0 && ep > 0) {
      retrainMutation.mutate({ training_examples: examples, epochs: ep });
    }
  };

  if (statusError) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-red-800">
            <AlertTriangle className="h-5 w-5" />
            <span className="font-medium">Failed to load drift detection data</span>
          </div>
          <p className="mt-2 text-sm text-red-700">
            {statusError instanceof Error ? statusError.message : 'Unknown error'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 p-6">
        <div className="flex items-center gap-3">
          <TrendingDown className="h-6 w-6 text-yellow-600" />
          <h2 className="text-2xl font-bold text-gray-900">Model Drift Detection</h2>
        </div>
        <p className="mt-1 text-sm text-gray-600">
          Monitor model accuracy degradation and trigger retraining
        </p>
      </div>

      {/* Current Status */}
      {status && (
        <div className="p-6 bg-white border-b border-gray-200">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Status Overview */}
            <div className="bg-gray-50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
                <Activity className="h-5 w-5" />
                Current Status
              </h3>

              <div className="space-y-3">
                <div>
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm text-gray-600">Drift Level</span>
                    {getDriftLevelBadge(status.drift_level)}
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-600">Current Accuracy</span>
                    <span className={`font-medium ${
                      status.drift_level === 'critical' ? 'text-red-600' :
                      status.drift_level === 'warning' ? 'text-yellow-600' :
                      'text-green-600'
                    }`}>
                      {(status.current_accuracy * 100).toFixed(2)}%
                    </span>
                  </div>
                  <div className="bg-gray-200 rounded-full h-2">
                    <div
                      className={`h-2 rounded-full ${
                        status.drift_level === 'critical' ? 'bg-red-600' :
                        status.drift_level === 'warning' ? 'bg-yellow-600' :
                        'bg-green-600'
                      }`}
                      style={{ width: `${status.current_accuracy * 100}%` }}
                    />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-sm mb-1">
                    <span className="text-gray-600">Baseline Accuracy</span>
                    <span className="font-medium text-gray-900">
                      {(status.baseline_accuracy * 100).toFixed(2)}%
                    </span>
                  </div>
                  <div className="bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-blue-600 h-2 rounded-full"
                      style={{ width: `${status.baseline_accuracy * 100}%` }}
                    />
                  </div>
                </div>

                <div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-600">Accuracy Delta</span>
                    <span className={`font-medium ${status.accuracy_delta < 0 ? 'text-red-600' : 'text-green-600'}`}>
                      {status.accuracy_delta > 0 ? '+' : ''}{(status.accuracy_delta * 100).toFixed(2)}%
                    </span>
                  </div>
                </div>

                <div className="pt-3 border-t border-gray-200 space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Sample Window Size:</span>
                    <span className="font-medium text-gray-900">{status.sample_window_size}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">Samples Analyzed:</span>
                    <span className="font-medium text-gray-900">{status.samples_analyzed.toLocaleString()}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">Last Retrain:</span>
                    <span className="font-medium text-gray-900">
                      {new Date(status.last_retrain).toLocaleString()}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            {/* Retrain Controls */}
            <div className="bg-gray-50 rounded-lg p-4">
              <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
                <RotateCcw className="h-5 w-5" />
                Retraining Controls
              </h3>

              <div className="space-y-4">
                <div>
                  <label htmlFor="training-examples" className="block text-sm text-gray-700 mb-1">
                    Training Examples
                  </label>
                  <input
                    id="training-examples"
                    type="number"
                    min="100"
                    max="10000"
                    step="100"
                    value={trainingExamples}
                    onChange={(e) => setTrainingExamples(e.target.value)}
                    className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500"
                  />
                </div>

                <div>
                  <label htmlFor="epochs" className="block text-sm text-gray-700 mb-1">
                    Training Epochs
                  </label>
                  <input
                    id="epochs"
                    type="number"
                    min="1"
                    max="100"
                    value={epochs}
                    onChange={(e) => setEpochs(e.target.value)}
                    className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-yellow-500"
                  />
                </div>

                <button
                  onClick={handleRetrain}
                  disabled={retrainMutation.isPending}
                  className="w-full px-4 py-2 bg-yellow-600 text-white rounded-md hover:bg-yellow-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Play className="h-4 w-4" />
                  {retrainMutation.isPending ? 'Retraining...' : 'Trigger Manual Retrain'}
                </button>

                <button
                  onClick={() => resetBaselineMutation.mutate()}
                  disabled={resetBaselineMutation.isPending}
                  className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                  <Target className="h-4 w-4" />
                  Reset Baseline to Current
                </button>

                <div className="pt-3 border-t border-gray-200">
                  <label className="flex items-center gap-2">
                    <input
                      type="checkbox"
                      checked={status.auto_retrain_enabled}
                      disabled
                      className="w-4 h-4 text-yellow-600 border-gray-300 rounded focus:ring-yellow-500"
                    />
                    <span className="text-sm text-gray-700">
                      Auto-retrain enabled (threshold: {(status.auto_retrain_threshold * 100).toFixed(1)}%)
                    </span>
                  </label>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Accuracy Trend Chart */}
      {chartData && status && (
        <div className="p-6 bg-white border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Accuracy Trend (Last {chartData.samples.length} samples)</h3>

          <div className="bg-gray-50 rounded-lg p-4" style={{ height: '240px' }}>
            <div className="relative h-full">
              {/* Y-axis labels */}
              <div className="absolute left-0 top-0 bottom-0 w-12 flex flex-col justify-between text-xs text-gray-600">
                <span>{(chartData.maxAccuracy * 100).toFixed(1)}%</span>
                <span>{((chartData.maxAccuracy + chartData.minAccuracy) / 2 * 100).toFixed(1)}%</span>
                <span>{(chartData.minAccuracy * 100).toFixed(1)}%</span>
              </div>

              {/* Chart area */}
              <div className="absolute left-12 right-0 top-0 bottom-0">
                {/* Baseline line */}
                <div
                  className="absolute left-0 right-0 border-t-2 border-dashed border-blue-400"
                  style={{
                    top: `${((chartData.maxAccuracy - status.baseline_accuracy) / chartData.range) * 100}%`,
                  }}
                >
                  <span className="absolute right-0 -top-2 text-xs text-blue-600 bg-white px-1">
                    Baseline
                  </span>
                </div>

                {/* Data points and line */}
                <svg className="w-full h-full">
                  {/* Line path */}
                  <polyline
                    points={chartData.samples
                      .map((sample, idx) => {
                        const x = (idx / (chartData.samples.length - 1)) * 100;
                        const y = ((chartData.maxAccuracy - sample.accuracy) / chartData.range) * 100;
                        return `${x}%,${y}%`;
                      })
                      .join(' ')}
                    fill="none"
                    stroke={
                      status.drift_level === 'critical' ? '#dc2626' :
                      status.drift_level === 'warning' ? '#d97706' :
                      '#16a34a'
                    }
                    strokeWidth="2"
                  />

                  {/* Data points */}
                  {chartData.samples.map((sample, idx) => {
                    const x = (idx / (chartData.samples.length - 1)) * 100;
                    const y = ((chartData.maxAccuracy - sample.accuracy) / chartData.range) * 100;
                    const color =
                      sample.drift_level === 'critical' ? '#dc2626' :
                      sample.drift_level === 'warning' ? '#d97706' :
                      '#16a34a';

                    return (
                      <circle
                        key={idx}
                        cx={`${x}%`}
                        cy={`${y}%`}
                        r="3"
                        fill={color}
                      />
                    );
                  })}
                </svg>
              </div>
            </div>
          </div>

          <div className="mt-2 flex items-center justify-center gap-4 text-xs text-gray-600">
            <div className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-green-600"></div>
              Normal
            </div>
            <div className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-yellow-600"></div>
              Warning
            </div>
            <div className="flex items-center gap-1">
              <div className="w-3 h-3 rounded-full bg-red-600"></div>
              Critical
            </div>
            <div className="flex items-center gap-1">
              <div className="w-6 h-0 border-t-2 border-dashed border-blue-400"></div>
              Baseline
            </div>
          </div>
        </div>
      )}

      {/* Drift History Table */}
      <div className="flex-1 flex flex-col p-6 overflow-hidden">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Drift History</h3>

        <div
          ref={tableRef}
          className="flex-1 overflow-auto bg-white border border-gray-200 rounded-lg"
          style={{ contain: 'strict' }}
        >
          <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const sample = samples[virtualRow.index];
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
                  className="border-b border-gray-100 hover:bg-gray-50 px-4 py-3 flex items-center justify-between"
                >
                  <div className="flex items-center gap-4">
                    <div className="text-xs text-gray-500 w-24">
                      {new Date(sample.timestamp).toLocaleTimeString()}
                    </div>
                    {getDriftLevelBadge(sample.drift_level)}
                    <div className="text-sm text-gray-900">
                      Accuracy: <span className="font-medium">{(sample.accuracy * 100).toFixed(2)}%</span>
                    </div>
                    <div className="text-sm text-gray-600">
                      Delta: <span className={`font-medium ${sample.accuracy_delta < 0 ? 'text-red-600' : 'text-green-600'}`}>
                        {sample.accuracy_delta > 0 ? '+' : ''}{(sample.accuracy_delta * 100).toFixed(2)}%
                      </span>
                    </div>
                  </div>
                  <div className="text-xs text-gray-500">
                    Samples: {sample.sample_count.toLocaleString()}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {samples.length === 0 && (
          <div className="flex-1 flex items-center justify-center bg-white border border-gray-200 rounded-lg">
            <div className="text-center py-12">
              <Clock className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">No drift samples yet</p>
              <p className="text-sm text-gray-500 mt-1">
                Drift detection history will appear here as the model processes requests
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
