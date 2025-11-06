/**
 * Dashboard with status cards and default metric charts
 */

import { useQuery, useQueries } from '@tanstack/react-query';
import { LineChart, Line, ResponsiveContainer, YAxis, XAxis } from 'recharts';
import { Activity, Terminal, PlayCircle, Brain } from 'lucide-react';
import { qemuApi, metricsApi, replayApi, autonomyApi, QemuState, ReplayState } from '../lib/api';

const DEFAULT_METRICS = [
  { name: 'nn_infer_us', label: 'NN Inference (μs)', color: '#10b981' },
  { name: 'irq_latency_ns', label: 'IRQ Latency (ns)', color: '#ef4444' },
  { name: 'memory_alloc_ns', label: 'Memory Alloc (ns)', color: '#3b82f6' },
  { name: 'real_ctx_switch_ns', label: 'Context Switch (ns)', color: '#f59e0b' },
];

interface DashboardProps {
  daemonHealthy: boolean;
}

export function Dashboard({ daemonHealthy }: DashboardProps) {
  // Fetch QEMU status
  const { data: qemuStatus } = useQuery({
    queryKey: ['qemu-status'],
    queryFn: () => qemuApi.status(),
    enabled: daemonHealthy,
    refetchInterval: 2000,
  });

  // Fetch replay status
  const { data: replayStatus } = useQuery({
    queryKey: ['replay-status'],
    queryFn: () => replayApi.status(),
    enabled: daemonHealthy,
    refetchInterval: 2000,
  });

  // Fetch autonomy status
  const { data: autonomyStatus } = useQuery({
    queryKey: ['autonomy', 'status'],
    queryFn: () => autonomyApi.status(),
    enabled: daemonHealthy,
    refetchInterval: 2000,
    retry: false,
  });

  // Fetch default metrics
  const metricsQueries = useQueries({
    queries: DEFAULT_METRICS.map((metric) => ({
      queryKey: ['metrics', 'query', metric.name],
      queryFn: () => {
        const now = Date.now();
        return metricsApi.query({
          name: metric.name,
          from: now - 5 * 60 * 1000, // Last 5 minutes
          to: now,
          maxPoints: 100,
        });
      },
      enabled: daemonHealthy && qemuStatus?.state === QemuState.Running,
      refetchInterval: 2000,
      retry: false,
    })),
  });

  // Status cards
  const cards = [
    {
      icon: Activity,
      label: 'QEMU',
      value: qemuStatus?.state || 'idle',
      color:
        qemuStatus?.state === QemuState.Running
          ? 'text-green-500'
          : qemuStatus?.state === QemuState.Failed
          ? 'text-red-500'
          : 'text-muted-foreground',
    },
    {
      icon: Terminal,
      label: 'Shell',
      value: qemuStatus?.state === QemuState.Running ? 'ready' : 'not ready',
      color:
        qemuStatus?.state === QemuState.Running
          ? 'text-green-500'
          : 'text-muted-foreground',
    },
    {
      icon: PlayCircle,
      label: 'Replay',
      value: replayStatus?.state || 'idle',
      color:
        replayStatus?.state === ReplayState.Running
          ? 'text-blue-500'
          : 'text-muted-foreground',
    },
    {
      icon: Brain,
      label: 'Autonomy',
      value: autonomyStatus?.enabled
        ? `${autonomyStatus.total_decisions} decisions`
        : 'disabled',
      color: autonomyStatus?.enabled
        ? 'text-green-500'
        : 'text-muted-foreground',
      detail: autonomyStatus?.enabled
        ? `${autonomyStatus.accepted} acc / ${autonomyStatus.deferred} def`
        : undefined,
    },
  ];

  return (
    <div className="space-y-4">
      {/* Status Cards */}
      <div className="grid grid-cols-4 gap-3">
        {cards.map((card, idx) => (
          <div
            key={idx}
            className="bg-card rounded-lg border p-4 flex flex-col items-center justify-center"
          >
            <card.icon className={`h-6 w-6 mb-2 ${card.color}`} />
            <div className="text-xs text-muted-foreground mb-1">
              {card.label}
            </div>
            <div className={`text-sm font-semibold ${card.color}`}>
              {card.value}
            </div>
            {'detail' in card && card.detail && (
              <div className="text-xs text-muted-foreground mt-1">
                {card.detail}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Default Charts */}
      <div className="grid grid-cols-2 gap-3">
        {DEFAULT_METRICS.map((metric, idx) => {
          const queryResult = metricsQueries[idx].data;
          const hasData = queryResult && queryResult.points.length > 0;

          return (
            <div key={metric.name} className="bg-card rounded-lg border p-4">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-medium text-muted-foreground">
                  {metric.label}
                </h4>
                {hasData && (
                  <span className="text-lg font-bold">
                    {queryResult.points[queryResult.points.length - 1].value.toFixed(
                      0
                    )}
                  </span>
                )}
              </div>

              <div className="h-24">
                {hasData ? (
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={queryResult.points}>
                      <XAxis
                        dataKey="ts"
                        hide
                        domain={['auto', 'auto']}
                      />
                      <YAxis hide domain={['auto', 'auto']} />
                      <Line
                        type="monotone"
                        dataKey="value"
                        stroke={metric.color}
                        strokeWidth={2}
                        dot={false}
                        isAnimationActive={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                ) : (
                  <div className="h-full flex items-center justify-center text-xs text-muted-foreground">
                    {qemuStatus?.state === QemuState.Running
                      ? 'Waiting for data...'
                      : 'Start QEMU to see metrics'}
                  </div>
                )}
              </div>

              {hasData && (
                <div className="mt-2 text-xs text-muted-foreground">
                  {queryResult.points.length} points
                  {queryResult.downsampled && ' (downsampled)'}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
