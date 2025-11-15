/**
 * What-if scenario simulator with baseline vs scenario comparison
 */

import { useState, useEffect, useRef } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { autonomyApi, PreviewResponse } from '../lib/api';
import { GitCompare, Download, AlertTriangle, Check, X } from 'lucide-react';

export function WhatIfSimulator() {
  const [overrides, setOverrides] = useState({
    mem: 0,
    frag: 0,
    misses: 0,
    cmd_rate: 0,
  });
  const debounceTimerRef = useRef<NodeJS.Timeout>();

  // Fetch baseline preview
  const { data: baseline } = useQuery({
    queryKey: ['autonomy', 'preview'],
    queryFn: () => autonomyApi.preview(10),
    refetchInterval: 5000,
  });

  // What-if mutation with debounce
  const whatIf = useMutation({
    mutationFn: (overrides: Record<string, any>) => autonomyApi.whatIf(overrides),
  });

  // Debounced what-if query
  useEffect(() => {
    // Clear existing timer
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
    }

    // Set new timer
    debounceTimerRef.current = setTimeout(() => {
      // Only query if we have some overrides
      const hasOverrides = Object.values(overrides).some(v => v !== 0);
      if (hasOverrides) {
        whatIf.mutate(overrides);
      }
    }, 300);

    return () => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
    };
  }, [overrides]);

  // Export JSON
  const exportJSON = () => {
    const data = {
      baseline: baseline || null,
      scenario: whatIf.data?.scenario || null,
      overrides,
      timestamp: new Date().toISOString(),
    };

    const blob = new Blob([JSON.stringify(data, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `whatif_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Copy scenario JSON
  const copyScenario = () => {
    const data = {
      overrides,
      result: whatIf.data?.scenario || null,
    };
    navigator.clipboard.writeText(JSON.stringify(data, null, 2));
  };

  const scenario = whatIf.data?.scenario;

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="p-4 border-b flex items-center justify-between">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <GitCompare className="h-5 w-5" />
          What-If Simulator
        </h3>
        <div className="flex gap-2">
          <button
            onClick={copyScenario}
            disabled={!scenario}
            className="px-3 py-1.5 text-sm bg-muted hover:bg-muted/80 rounded disabled:opacity-50"
          >
            Copy JSON
          </button>
          <button
            onClick={exportJSON}
            className="px-3 py-1.5 text-sm bg-primary text-primary-foreground hover:bg-primary/90 rounded flex items-center gap-1"
          >
            <Download className="h-4 w-4" />
            Export
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {/* Controls */}
        <div>
          <div className="text-sm font-semibold mb-3">Scenario Parameters</div>
          <div className="grid grid-cols-2 gap-4">
            {/* Memory Pressure */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Memory Pressure (%)
              </label>
              <input
                type="range"
                value={overrides.mem}
                onChange={(e) =>
                  setOverrides({ ...overrides, mem: parseInt(e.target.value) })
                }
                className="w-full"
                min="0"
                max="100"
              />
              <div className="text-xs text-right mt-1 font-mono">{overrides.mem}%</div>
            </div>

            {/* Fragmentation */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Fragmentation (%)
              </label>
              <input
                type="range"
                value={overrides.frag}
                onChange={(e) =>
                  setOverrides({ ...overrides, frag: parseInt(e.target.value) })
                }
                className="w-full"
                min="0"
                max="100"
              />
              <div className="text-xs text-right mt-1 font-mono">{overrides.frag}%</div>
            </div>

            {/* Cache Misses */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Cache Misses (%)
              </label>
              <input
                type="range"
                value={overrides.misses}
                onChange={(e) =>
                  setOverrides({ ...overrides, misses: parseInt(e.target.value) })
                }
                className="w-full"
                min="0"
                max="100"
              />
              <div className="text-xs text-right mt-1 font-mono">{overrides.misses}%</div>
            </div>

            {/* Command Rate */}
            <div>
              <label className="text-xs text-muted-foreground mb-1 block">
                Command Rate (Hz)
              </label>
              <input
                type="range"
                value={overrides.cmd_rate}
                onChange={(e) =>
                  setOverrides({ ...overrides, cmd_rate: parseInt(e.target.value) })
                }
                className="w-full"
                min="0"
                max="1000"
                step="10"
              />
              <div className="text-xs text-right mt-1 font-mono">{overrides.cmd_rate} Hz</div>
            </div>
          </div>
        </div>

        {/* Comparison */}
        <div className="grid grid-cols-2 gap-4">
          {/* Baseline */}
          <div>
            <div className="text-sm font-semibold mb-2">Baseline (Current)</div>
            {baseline ? (
              <PreviewCard preview={baseline} isBaseline />
            ) : (
              <div className="bg-background rounded p-4 text-center text-sm text-muted-foreground">
                Loading baseline...
              </div>
            )}
          </div>

          {/* Scenario */}
          <div>
            <div className="text-sm font-semibold mb-2">Scenario (Modified)</div>
            {whatIf.isPending ? (
              <div className="bg-background rounded p-4 flex items-center justify-center">
                <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
              </div>
            ) : whatIf.error ? (
              <div className="bg-red-500/10 border border-red-500 rounded p-4 text-sm">
                <div className="font-semibold text-red-500 mb-1">Error</div>
                <div className="text-red-500/80 text-xs">
                  {whatIf.error instanceof Error ? whatIf.error.message : 'Failed to run scenario'}
                </div>
              </div>
            ) : scenario ? (
              <PreviewCard preview={scenario} isBaseline={false} />
            ) : (
              <div className="bg-background rounded p-4 text-center text-sm text-muted-foreground">
                Adjust parameters to see scenario
              </div>
            )}
          </div>
        </div>

        {/* Diff */}
        {whatIf.data?.diff && whatIf.data.diff.length > 0 && (
          <div>
            <div className="text-sm font-semibold mb-2">Changes</div>
            <div className="bg-background rounded p-3 space-y-1">
              {whatIf.data.diff.map((change, idx) => (
                <div key={idx} className="text-xs font-mono text-muted-foreground">
                  {change}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// Preview card component
function PreviewCard({
  preview,
  isBaseline,
}: {
  preview: PreviewResponse;
  isBaseline: boolean;
}) {
  return (
    <div className="bg-background rounded border p-4 space-y-3">
      {/* Would Execute Badge */}
      <div className="flex items-center justify-between">
        <span className="text-xs text-muted-foreground">
          {isBaseline ? 'Current' : 'Predicted'}
        </span>
        <span
          className={`px-2 py-1 rounded text-xs font-semibold flex items-center gap-1 ${
            preview.would_execute
              ? 'bg-green-500/10 text-green-500'
              : 'bg-red-500/10 text-red-500'
          }`}
        >
          {preview.would_execute ? (
            <>
              <Check className="h-3 w-3" />
              Would Execute
            </>
          ) : (
            <>
              <X className="h-3 w-3" />
              Would Defer
            </>
          )}
        </span>
      </div>

      {/* Confidence */}
      <div>
        <div className="text-xs text-muted-foreground mb-1">Confidence</div>
        <div className="flex items-center gap-2">
          <div className="flex-1 bg-muted rounded h-2 overflow-hidden">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${preview.confidence * 100}%` }}
            />
          </div>
          <span className="text-sm font-semibold">
            {(preview.confidence * 100).toFixed(1)}%
          </span>
        </div>
      </div>

      {/* Directives */}
      <div>
        <div className="text-xs text-muted-foreground mb-1">
          Directives ({preview.directives.length})
        </div>
        <div className="space-y-1 max-h-32 overflow-y-auto">
          {preview.directives.map((directive, idx) => (
            <div
              key={idx}
              className="text-xs font-mono bg-muted/50 rounded px-2 py-1"
            >
              {directive}
            </div>
          ))}
        </div>
      </div>

      {/* Warnings */}
      {preview.warnings.length > 0 && (
        <div>
          <div className="text-xs text-muted-foreground mb-1 flex items-center gap-1">
            <AlertTriangle className="h-3 w-3 text-yellow-500" />
            Warnings
          </div>
          <div className="space-y-1">
            {preview.warnings.map((warning, idx) => (
              <div
                key={idx}
                className="text-xs bg-yellow-500/10 text-yellow-500 rounded px-2 py-1"
              >
                {warning}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
