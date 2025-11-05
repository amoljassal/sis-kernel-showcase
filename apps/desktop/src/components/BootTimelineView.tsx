/**
 * Boot Timeline View - M4 Enhancement
 *
 * Visualizes boot sequence as a horizontal timeline with:
 * - Event markers
 * - Time deltas between events
 * - Interactive timeline scrubbing
 * - Performance insights
 */

import { useState, useEffect, useMemo } from 'react';
import { Clock, Zap, CheckCircle, Info } from 'lucide-react';

interface BootEvent {
  marker: string;
  label: string;
  timestamp: number;
  description?: string;
}

interface BootTimelineViewProps {
  markers: Record<string, boolean>;
  startTime?: number;
}

const BOOT_MARKER_LABELS: Record<string, string> = {
  KernelU: 'Kernel Entry',
  StackOk: 'Stack Init',
  MmuSctlr: 'MMU Config',
  MmuOn: 'MMU Enabled',
  UartReady: 'UART Ready',
  GicInit: 'GIC Init',
  VectorsOk: 'Vectors OK',
  LaunchingShell: 'Shell Launch',
  ShellReady: 'Shell Ready',
};

export function BootTimelineView({ markers, startTime }: BootTimelineViewProps) {
  const [events, setEvents] = useState<BootEvent[]>([]);
  const [selectedEvent, setSelectedEvent] = useState<BootEvent | null>(null);

  // Track marker timestamps
  useEffect(() => {
    Object.entries(markers).forEach(([marker, isComplete]) => {
      if (isComplete && !events.find((e) => e.marker === marker)) {
        setEvents((prev) => [
          ...prev,
          {
            marker,
            label: BOOT_MARKER_LABELS[marker] || marker,
            timestamp: Date.now(),
          },
        ].sort((a, b) => a.timestamp - b.timestamp));
      }
    });
  }, [markers]);

  // Calculate timeline metrics
  const timelineMetrics = useMemo(() => {
    if (events.length === 0) {
      return {
        totalDuration: 0,
        averageDelta: 0,
        minDelta: 0,
        maxDelta: 0,
      };
    }

    const deltas = events.slice(1).map((event, idx) => event.timestamp - events[idx].timestamp);
    const totalDuration = events[events.length - 1].timestamp - events[0].timestamp;

    return {
      totalDuration,
      averageDelta: deltas.length > 0 ? deltas.reduce((a, b) => a + b, 0) / deltas.length : 0,
      minDelta: deltas.length > 0 ? Math.min(...deltas) : 0,
      maxDelta: deltas.length > 0 ? Math.max(...deltas) : 0,
      deltas,
    };
  }, [events]);

  // Calculate position on timeline (0-100%)
  const getEventPosition = (event: BootEvent) => {
    if (events.length <= 1) return 0;
    const totalDuration = events[events.length - 1].timestamp - events[0].timestamp;
    if (totalDuration === 0) return 0;
    const eventOffset = event.timestamp - events[0].timestamp;
    return (eventOffset / totalDuration) * 100;
  };

  // Format duration
  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(2)}s`;
    return `${(ms / 60000).toFixed(2)}m`;
  };

  if (events.length === 0) {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground">
        <div className="text-center">
          <Clock className="h-12 w-12 mx-auto mb-4 opacity-50" />
          <p>No boot events recorded yet</p>
          <p className="text-sm mt-2">Start QEMU to see boot timeline</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-6 gap-6">
      {/* Header with metrics */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold flex items-center gap-2">
            <Zap className="h-6 w-6 text-primary" />
            Boot Timeline
          </h2>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Clock className="h-4 w-4" />
            Total: {formatDuration(timelineMetrics.totalDuration)}
          </div>
        </div>

        {/* Performance metrics */}
        <div className="grid grid-cols-4 gap-4">
          <div className="bg-muted rounded-md p-3">
            <div className="text-xs text-muted-foreground mb-1">Events</div>
            <div className="text-2xl font-bold">{events.length}</div>
          </div>
          <div className="bg-muted rounded-md p-3">
            <div className="text-xs text-muted-foreground mb-1">Avg Delta</div>
            <div className="text-2xl font-bold">{formatDuration(timelineMetrics.averageDelta)}</div>
          </div>
          <div className="bg-muted rounded-md p-3">
            <div className="text-xs text-muted-foreground mb-1">Min Delta</div>
            <div className="text-2xl font-bold text-green-600">{formatDuration(timelineMetrics.minDelta)}</div>
          </div>
          <div className="bg-muted rounded-md p-3">
            <div className="text-xs text-muted-foreground mb-1">Max Delta</div>
            <div className="text-2xl font-bold text-orange-600">{formatDuration(timelineMetrics.maxDelta)}</div>
          </div>
        </div>
      </div>

      {/* Timeline visualization */}
      <div className="flex-1 flex flex-col gap-4">
        {/* Timeline track */}
        <div className="relative h-32 bg-muted rounded-lg p-4">
          {/* Timeline bar */}
          <div className="absolute top-1/2 left-4 right-4 h-2 bg-primary/20 rounded-full">
            {/* Progress bar */}
            <div
              className="h-full bg-primary rounded-full transition-all duration-300"
              style={{ width: '100%' }}
            />
          </div>

          {/* Event markers */}
          {events.map((event, idx) => {
            const position = getEventPosition(event);
            const isSelected = selectedEvent?.marker === event.marker;

            return (
              <div
                key={event.marker}
                className="absolute"
                style={{
                  left: `${4 + (position * 0.92)}%`,
                  top: '50%',
                  transform: 'translate(-50%, -50%)',
                }}
              >
                {/* Marker dot */}
                <button
                  onClick={() => setSelectedEvent(event)}
                  className={`w-4 h-4 rounded-full transition-all ${
                    isSelected
                      ? 'bg-primary ring-4 ring-primary/30 scale-150'
                      : 'bg-primary hover:bg-primary/80 hover:scale-125'
                  }`}
                  title={event.label}
                />

                {/* Label */}
                <div
                  className={`absolute top-6 left-1/2 -translate-x-1/2 text-xs whitespace-nowrap transition-all ${
                    isSelected ? 'font-semibold text-primary' : 'text-muted-foreground'
                  }`}
                >
                  {event.label}
                </div>

                {/* Time since previous */}
                {idx > 0 && (
                  <div className="absolute -top-6 left-1/2 -translate-x-1/2 text-[10px] text-muted-foreground whitespace-nowrap">
                    +{formatDuration(event.timestamp - events[idx - 1].timestamp)}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {/* Event details list */}
        <div className="flex-1 overflow-y-auto space-y-2">
          <h3 className="text-sm font-semibold text-muted-foreground mb-2 flex items-center gap-2">
            <Info className="h-4 w-4" />
            Event Sequence
          </h3>
          {events.map((event, idx) => {
            const isSelected = selectedEvent?.marker === event.marker;
            const delta = idx > 0 ? event.timestamp - events[idx - 1].timestamp : 0;
            const isSlowStep = delta > timelineMetrics.averageDelta * 1.5;

            return (
              <button
                key={event.marker}
                onClick={() => setSelectedEvent(event)}
                className={`w-full text-left p-3 rounded-md border transition-all ${
                  isSelected
                    ? 'bg-primary/10 border-primary'
                    : 'bg-muted/50 border-border hover:bg-muted'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    <CheckCircle className="h-5 w-5 text-green-600" />
                    <div>
                      <div className="font-medium">{event.label}</div>
                      <div className="text-xs text-muted-foreground">
                        {new Date(event.timestamp).toLocaleTimeString()}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    {idx > 0 && (
                      <div
                        className={`text-sm font-mono ${
                          isSlowStep ? 'text-orange-600 font-semibold' : 'text-muted-foreground'
                        }`}
                      >
                        +{formatDuration(delta)}
                      </div>
                    )}
                    <div className="text-xs text-muted-foreground">
                      #{idx + 1}
                    </div>
                  </div>
                </div>
              </button>
            );
          })}
        </div>

        {/* Performance insights */}
        {timelineMetrics.maxDelta > timelineMetrics.averageDelta * 2 && (
          <div className="bg-orange-500/10 border border-orange-500/20 rounded-md p-3 text-sm">
            <div className="flex items-start gap-2">
              <Info className="h-4 w-4 text-orange-600 mt-0.5 flex-shrink-0" />
              <div>
                <div className="font-semibold text-orange-900 dark:text-orange-100">
                  Performance Note
                </div>
                <div className="text-orange-800 dark:text-orange-200 mt-1">
                  Some boot steps are taking significantly longer than average. Check kernel logs for potential issues.
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
