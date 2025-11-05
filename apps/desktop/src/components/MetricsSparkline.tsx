/**
 * Simple metrics sparkline visualization
 */

import { useEffect, useState } from 'react';
import { LineChart, Line, ResponsiveContainer } from 'recharts';

interface MetricPoint {
  timestamp: number;
  value: number;
}

interface MetricsSparklineProps {
  metrics: MetricPoint[];
  title: string;
  color?: string;
}

export function MetricsSparkline({
  metrics,
  title,
  color = '#3b82f6',
}: MetricsSparklineProps) {
  const [data, setData] = useState<MetricPoint[]>([]);

  useEffect(() => {
    // Keep last 50 points
    setData(metrics.slice(-50));
  }, [metrics]);

  const latestValue = data.length > 0 ? data[data.length - 1].value : 0;

  return (
    <div className="bg-card rounded-lg border p-4">
      <div className="flex items-center justify-between mb-2">
        <h4 className="text-sm font-medium text-muted-foreground">{title}</h4>
        <span className="text-2xl font-bold">{latestValue.toFixed(1)}</span>
      </div>

      <div className="h-16">
        {data.length > 0 ? (
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={data}>
              <Line
                type="monotone"
                dataKey="value"
                stroke={color}
                strokeWidth={2}
                dot={false}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <div className="h-full flex items-center justify-center text-muted-foreground text-sm">
            No data
          </div>
        )}
      </div>

      <div className="mt-2 text-xs text-muted-foreground">
        {data.length} points
      </div>
    </div>
  );
}
