/**
 * Comprehensive metrics panel with virtualized list, sparklines, and export
 */

import { useState, useRef, useMemo, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  metricsApi,
  MetricPoint,
  BatchedMetricPoint,
} from '../lib/api';
import { LineChart, Line, ResponsiveContainer, YAxis } from 'recharts';
import {
  Download,
  Pause,
  Play,
  Search,
  TrendingUp,
  X,
} from 'lucide-react';

type TimeRange = '5m' | '30m' | '1h';

interface MetricsState {
  [seriesName: string]: MetricPoint[];
}

interface MetricsPanelProps {
  /** Incoming metric batch from WebSocket */
  metricBatch?: {
    points: BatchedMetricPoint[];
    seq?: number;
    droppedCount?: number;
  } | null;
}

export function MetricsPanel({ metricBatch }: MetricsPanelProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedSeries, setSelectedSeries] = useState<string | null>(null);
  const [timeRange, setTimeRange] = useState<TimeRange>('5m');
  const [isPaused, setIsPaused] = useState(false);
  const [metricsData, setMetricsData] = useState<MetricsState>({});
  const [lastSeq, setLastSeq] = useState<number>(0);
  const [droppedCount, setDroppedCount] = useState<number>(0);

  const parentRef = useRef<HTMLDivElement>(null);

  // Convert time range to milliseconds
  const timeRangeMs = useMemo(() => {
    switch (timeRange) {
      case '5m':
        return 5 * 60 * 1000;
      case '30m':
        return 30 * 60 * 1000;
      case '1h':
        return 60 * 60 * 1000;
    }
  }, [timeRange]);

  // Fetch series list
  const { data: seriesList = [], refetch: refetchSeriesList } = useQuery({
    queryKey: ['metrics', 'streams', searchQuery],
    queryFn: () =>
      metricsApi.listStreams(searchQuery ? { prefix: searchQuery } : undefined),
    enabled: !isPaused,
    refetchInterval: isPaused ? false : 5000,
  });

  // Fetch selected series data
  const { data: selectedData, refetch: refetchSelectedData } = useQuery({
    queryKey: ['metrics', 'query', selectedSeries, timeRange],
    queryFn: () => {
      if (!selectedSeries) return null;
      const now = Date.now();
      return metricsApi.query({
        name: selectedSeries,
        from: now - timeRangeMs,
        to: now,
        maxPoints: 1000,
      });
    },
    enabled: !isPaused && selectedSeries !== null,
    refetchInterval: isPaused ? false : 1000,
  });

  // Handle incoming metric batch from WebSocket
  useEffect(() => {
    if (isPaused || !metricBatch) return;

    const { points, seq, droppedCount: dropped } = metricBatch;

    // Check for seq gaps
    if (seq !== undefined && lastSeq > 0 && seq !== lastSeq + 1) {
      console.warn(`Metric batch seq gap detected: ${lastSeq} → ${seq}`);
      // Schedule full refresh
      setTimeout(() => {
        refetchSeriesList();
        if (selectedSeries) refetchSelectedData();
      }, 100);
    }

    if (seq !== undefined) setLastSeq(seq);
    if (dropped !== undefined && dropped > 0) {
      setDroppedCount((prev) => prev + dropped);
    }

    // Merge points into state (deduplicate by ts)
    setMetricsData((prev) => {
      const next = { ...prev };
      for (const point of points) {
        if (!next[point.name]) next[point.name] = [];
        const existing = next[point.name];
        const isDuplicate = existing.some((p) => p.ts === point.ts);
        if (!isDuplicate) {
          next[point.name] = [
            ...existing,
            { ts: point.ts, value: point.value },
          ].sort((a, b) => a.ts - b.ts);
        }
      }
      return next;
    });
  }, [
    isPaused,
    lastSeq,
    metricBatch,
    refetchSeriesList,
    refetchSelectedData,
    selectedSeries,
  ]);

  // Handle pause/resume
  const togglePause = () => {
    const newPaused = !isPaused;
    setIsPaused(newPaused);
    if (!newPaused) {
      // Resume: refresh from REST
      refetchSeriesList();
      if (selectedSeries) refetchSelectedData();
      setLastSeq(0); // Reset seq tracking
    }
  };

  // Virtualized list
  const rowVirtualizer = useVirtualizer({
    count: seriesList.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 5,
  });

  // Calculate delta for each series
  const seriesWithDelta = useMemo(() => {
    return seriesList.map((series) => {
      const lastValue = series.stats.last_value;
      const points = metricsData[series.name] || [];
      const prevValue =
        points.length >= 2 ? points[points.length - 2].value : lastValue;
      const delta = lastValue - prevValue;
      return { ...series, delta };
    });
  }, [seriesList, metricsData]);

  // Export functions
  const exportCSV = () => {
    if (!selectedSeries || !selectedData) return;
    const csv = [
      'ts,value',
      ...selectedData.points.map((p) => `${p.ts},${p.value}`),
    ].join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${selectedSeries}_${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const exportJSON = () => {
    if (!selectedSeries || !selectedData) return;
    const json = JSON.stringify(
      {
        name: selectedSeries,
        points: selectedData.points,
      },
      null,
      2
    );
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${selectedSeries}_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="p-4 border-b space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Metrics
          </h3>
          <button
            onClick={togglePause}
            className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
              isPaused
                ? 'bg-green-500/10 text-green-500 hover:bg-green-500/20'
                : 'bg-yellow-500/10 text-yellow-500 hover:bg-yellow-500/20'
            }`}
          >
            {isPaused ? (
              <Play className="h-4 w-4 inline mr-1" />
            ) : (
              <Pause className="h-4 w-4 inline mr-1" />
            )}
            {isPaused ? 'Resume' : 'Pause'}
          </button>
        </div>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Filter by name prefix..."
            className="w-full pl-9 pr-8 py-2 bg-background border rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-primary"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              <X className="h-4 w-4" />
            </button>
          )}
        </div>

        {/* Range selector */}
        <div className="flex gap-2">
          {(['5m', '30m', '1h'] as TimeRange[]).map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${
                timeRange === range
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground hover:bg-muted/80'
              }`}
            >
              {range}
            </button>
          ))}
        </div>

        {droppedCount > 0 && (
          <div className="text-xs text-yellow-500 bg-yellow-500/10 px-2 py-1 rounded">
            {droppedCount} points dropped due to backpressure
          </div>
        )}
      </div>

      {/* Series list (virtualized) */}
      <div
        ref={parentRef}
        className="flex-1 overflow-y-auto"
        style={{ contain: 'strict' }}
      >
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const series = seriesWithDelta[virtualRow.index];
            const isSelected = selectedSeries === series.name;
            const lastTsDate = new Date(series.lastTs);

            return (
              <div
                key={virtualRow.key}
                onClick={() => setSelectedSeries(series.name)}
                className={`absolute top-0 left-0 w-full px-4 py-3 cursor-pointer transition-colors border-b ${
                  isSelected
                    ? 'bg-primary/10 border-primary'
                    : 'hover:bg-muted/50'
                }`}
                style={{
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <div className="flex items-center justify-between text-sm">
                  <div className="font-medium truncate mr-2">{series.name}</div>
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <div className="font-mono">
                      {series.stats.last_value.toFixed(1)}
                    </div>
                    <div
                      className={`font-mono ${
                        series.delta > 0
                          ? 'text-green-500'
                          : series.delta < 0
                          ? 'text-red-500'
                          : ''
                      }`}
                    >
                      {series.delta > 0 ? '+' : ''}
                      {series.delta.toFixed(1)}
                    </div>
                    <div className="text-xs">
                      {lastTsDate.toLocaleTimeString()}
                    </div>
                  </div>
                </div>

                {/* Mini sparkline */}
                <div className="h-8 mt-1">
                  {metricsData[series.name]?.length > 0 && (
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={metricsData[series.name].slice(-20)}>
                        <Line
                          type="monotone"
                          dataKey="value"
                          stroke={isSelected ? '#3b82f6' : '#6b7280'}
                          strokeWidth={1}
                          dot={false}
                          isAnimationActive={false}
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Selected series detail */}
      {selectedSeries && selectedData && (
        <div className="border-t p-4 space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-semibold">{selectedSeries}</h4>
              <div className="text-xs text-muted-foreground">
                {selectedData.points.length} points
                {selectedData.downsampled && ' (downsampled)'}
              </div>
            </div>
            <div className="flex gap-2">
              <button
                onClick={exportCSV}
                className="px-2 py-1 text-xs bg-muted hover:bg-muted/80 rounded flex items-center gap-1"
              >
                <Download className="h-3 w-3" />
                CSV
              </button>
              <button
                onClick={exportJSON}
                className="px-2 py-1 text-xs bg-muted hover:bg-muted/80 rounded flex items-center gap-1"
              >
                <Download className="h-3 w-3" />
                JSON
              </button>
              <button
                onClick={() => setSelectedSeries(null)}
                className="px-2 py-1 text-xs bg-muted hover:bg-muted/80 rounded"
              >
                <X className="h-3 w-3" />
              </button>
            </div>
          </div>

          {/* Full chart */}
          <div className="h-40">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={selectedData.points}>
                <YAxis
                  domain={['auto', 'auto']}
                  width={60}
                  tick={{ fontSize: 10 }}
                />
                <Line
                  type="monotone"
                  dataKey="value"
                  stroke="#3b82f6"
                  strokeWidth={2}
                  dot={false}
                  isAnimationActive={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Stats */}
          <div className="grid grid-cols-4 gap-2 text-xs">
            {seriesList
              .find((s) => s.name === selectedSeries)
              ?.stats && (
              <>
                <div>
                  <div className="text-muted-foreground">Min</div>
                  <div className="font-mono">
                    {seriesList
                      .find((s) => s.name === selectedSeries)
                      ?.stats.min.toFixed(1)}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Max</div>
                  <div className="font-mono">
                    {seriesList
                      .find((s) => s.name === selectedSeries)
                      ?.stats.max.toFixed(1)}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Mean</div>
                  <div className="font-mono">
                    {seriesList
                      .find((s) => s.name === selectedSeries)
                      ?.stats.mean.toFixed(1)}
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground">Count</div>
                  <div className="font-mono">
                    {
                      seriesList.find((s) => s.name === selectedSeries)?.stats
                        .count
                    }
                  </div>
                </div>
              </>
            )}
          </div>
        </div>
      )}

      {/* Empty state */}
      {seriesList.length === 0 && (
        <div className="flex-1 flex items-center justify-center text-muted-foreground">
          {searchQuery ? 'No matching metrics' : 'No metrics available'}
        </div>
      )}
    </div>
  );
}
