/**
 * Decision explanation view with attention weights and importance bars
 */

import { useQuery } from '@tanstack/react-query';
import { autonomyApi, AutonomyDecision } from '../lib/api';
import { X, Info, TrendingUp } from 'lucide-react';
import { useEffect, useRef } from 'react';

interface ExplainViewProps {
  decision: AutonomyDecision;
  onClose: () => void;
}

export function ExplainView({ decision, onClose }: ExplainViewProps) {
  const firstBarRef = useRef<HTMLDivElement>(null);

  // Fetch explanation
  const { data: explanation, isLoading, error } = useQuery({
    queryKey: ['autonomy', 'explain', decision.id],
    queryFn: () => autonomyApi.explain(decision.id),
    retry: 2,
  });

  // Focus first bar on mount for keyboard navigation
  useEffect(() => {
    if (explanation && firstBarRef.current) {
      firstBarRef.current.focus();
    }
  }, [explanation]);

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent, index: number) => {
    if (!explanation) return;

    const bars = document.querySelectorAll('[role="progressbar"]');
    if (e.key === 'ArrowDown' && index < bars.length - 1) {
      e.preventDefault();
      (bars[index + 1] as HTMLElement).focus();
    } else if (e.key === 'ArrowUp' && index > 0) {
      e.preventDefault();
      (bars[index - 1] as HTMLElement).focus();
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-card border rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div className="p-4 border-b flex items-center justify-between sticky top-0 bg-card">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Info className="h-5 w-5" />
            Decision Explanation
          </h3>
          <button
            onClick={onClose}
            className="p-2 hover:bg-muted rounded"
            aria-label="Close explanation"
          >
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Loading */}
        {isLoading && (
          <div className="p-8 flex items-center justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          </div>
        )}

        {/* Error */}
        {error && (
          <div className="p-8">
            <div className="bg-red-500/10 border border-red-500 rounded p-4 text-sm">
              <div className="font-semibold text-red-500 mb-1">Failed to load explanation</div>
              <div className="text-red-500/80">
                {error instanceof Error ? error.message : 'Unknown error'}
              </div>
            </div>
          </div>
        )}

        {/* Content */}
        {explanation && (
          <div className="p-6 space-y-6">
            {/* Decision Summary */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <div className="text-xs text-muted-foreground mb-1">Decision ID</div>
                <div className="font-mono text-sm">{decision.id}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground mb-1">Timestamp</div>
                <div className="text-sm">{new Date(decision.timestamp).toLocaleString()}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground mb-1">Action</div>
                <div className="text-sm font-semibold">{explanation.action}</div>
              </div>
              <div>
                <div className="text-xs text-muted-foreground mb-1">Confidence</div>
                <div className="text-sm font-semibold">
                  {(explanation.confidence * 100).toFixed(1)}%
                </div>
              </div>
            </div>

            {/* Reasoning */}
            <div>
              <div className="text-sm font-semibold mb-2 flex items-center gap-2">
                <TrendingUp className="h-4 w-4" />
                Reasoning
              </div>
              <div className="bg-background rounded p-3 text-sm">
                {explanation.reasoning}
              </div>
            </div>

            {/* Attention Weights / Importance Bars */}
            <div>
              <div className="text-sm font-semibold mb-3">Feature Importance</div>
              <div className="space-y-4">
                {explanation.attention.map((weight, index) => {
                  const percentage = weight.weight * 100;
                  const valueId = `value-${decision.id}-${index}`;

                  return (
                    <div key={index}>
                      <div className="flex items-center justify-between mb-2">
                        <label
                          htmlFor={valueId}
                          className="text-sm font-medium"
                        >
                          {weight.feature}
                        </label>
                        <div className="text-sm text-muted-foreground">
                          {weight.value}
                        </div>
                      </div>

                      <div
                        ref={index === 0 ? firstBarRef : undefined}
                        role="progressbar"
                        aria-valuenow={percentage}
                        aria-valuemin={0}
                        aria-valuemax={100}
                        aria-describedby={valueId}
                        aria-label={`${weight.feature} importance: ${percentage.toFixed(1)}%`}
                        tabIndex={0}
                        onKeyDown={(e) => handleKeyDown(e, index)}
                        className="relative h-8 bg-muted rounded overflow-hidden focus:outline-none focus:ring-2 focus:ring-primary"
                      >
                        {/* Progress bar */}
                        <div
                          className="h-full bg-primary transition-all duration-300"
                          style={{ width: `${percentage}%` }}
                        >
                          <div className="h-full flex items-center justify-end pr-2">
                            <span className="text-xs font-semibold text-primary-foreground">
                              {percentage.toFixed(1)}%
                            </span>
                          </div>
                        </div>

                        {/* Hidden value for screen readers */}
                        <span id={valueId} className="sr-only">
                          Importance: {percentage.toFixed(1)}%, Value: {weight.value}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Keyboard hint */}
              <div className="mt-4 text-xs text-muted-foreground">
                Use <kbd className="px-1 py-0.5 bg-muted rounded">↑</kbd> and{' '}
                <kbd className="px-1 py-0.5 bg-muted rounded">↓</kbd> to navigate importance bars
              </div>
            </div>

            {/* Additional Context */}
            {decision.reason && (
              <div>
                <div className="text-sm font-semibold mb-2">Additional Context</div>
                <div className="bg-background rounded p-3 text-sm text-muted-foreground">
                  {decision.reason}
                </div>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
