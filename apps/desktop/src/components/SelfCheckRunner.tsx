/**
 * Self-check runner component
 */

import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import { shellApi, SelfCheckResponse } from '@/lib/api';
import { PlayCircle, CheckCircle, XCircle, Loader2, Clock } from 'lucide-react';

interface SelfCheckRunnerProps {
  disabled?: boolean;
}

export function SelfCheckRunner({ disabled }: SelfCheckRunnerProps) {
  const [lastResult, setLastResult] = useState<SelfCheckResponse | null>(null);

  const runSelfCheck = useMutation({
    mutationFn: () => shellApi.selfcheck(),
    onSuccess: (response) => {
      setLastResult(response);
    },
  });

  const handleRun = () => {
    if (disabled) return;
    runSelfCheck.mutate();
  };

  return (
    <div className="bg-card rounded-lg border p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Self-Check Tests</h3>
        <button
          onClick={handleRun}
          disabled={disabled || runSelfCheck.isPending}
          className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-2"
        >
          {runSelfCheck.isPending ? (
            <>
              <Loader2 className="h-4 w-4 animate-spin" />
              Running...
            </>
          ) : (
            <>
              <PlayCircle className="h-4 w-4" />
              Run Self-Check
            </>
          )}
        </button>
      </div>

      {lastResult && (
        <div className="space-y-4">
          {/* Summary */}
          <div
            className={`p-3 rounded-md ${
              lastResult.success
                ? 'bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800'
                : 'bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800'
            }`}
          >
            <div className="flex items-center gap-2 mb-2">
              {lastResult.success ? (
                <CheckCircle className="h-5 w-5 text-green-600 dark:text-green-400" />
              ) : (
                <XCircle className="h-5 w-5 text-red-600 dark:text-red-400" />
              )}
              <span
                className={`font-semibold ${
                  lastResult.success
                    ? 'text-green-900 dark:text-green-100'
                    : 'text-red-900 dark:text-red-100'
                }`}
              >
                {lastResult.success ? 'All tests passed!' : 'Some tests failed'}
              </span>
            </div>
            <div className="flex items-center gap-4 text-sm">
              <span className="text-muted-foreground">
                {lastResult.passed} / {lastResult.total} passed
              </span>
              <span className="flex items-center gap-1 text-muted-foreground">
                <Clock className="h-3 w-3" />
                {lastResult.execution_time_ms}ms
              </span>
            </div>
          </div>

          {/* Test Results */}
          <div className="space-y-2">
            <h4 className="text-sm font-medium text-muted-foreground">
              Test Results
            </h4>
            {lastResult.tests.map((test, index) => (
              <div
                key={index}
                className={`flex items-center gap-3 p-2 rounded ${
                  test.passed
                    ? 'bg-green-50 dark:bg-green-900/10'
                    : 'bg-red-50 dark:bg-red-900/10'
                }`}
              >
                {test.passed ? (
                  <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400 flex-shrink-0" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-600 dark:text-red-400 flex-shrink-0" />
                )}
                <span
                  className={`text-sm flex-1 ${
                    test.passed
                      ? 'text-green-900 dark:text-green-100'
                      : 'text-red-900 dark:text-red-100'
                  }`}
                >
                  {test.name}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {!lastResult && !runSelfCheck.isPending && (
        <div className="text-center py-8 text-muted-foreground">
          <p className="text-sm">
            Click "Run Self-Check" to test kernel functionality
          </p>
        </div>
      )}

      {runSelfCheck.error && (
        <div className="mt-4 p-3 bg-red-50 dark:bg-red-900/20 rounded-md border border-red-200 dark:border-red-800">
          <p className="text-sm text-red-900 dark:text-red-100">
            Error: {(runSelfCheck.error as any)?.response?.data?.error ||
              (runSelfCheck.error as any)?.message || 'Failed to run self-check'}
          </p>
        </div>
      )}
    </div>
  );
}
