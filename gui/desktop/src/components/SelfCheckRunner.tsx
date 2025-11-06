/**
 * Self-check runner component
 */

import { useState, useEffect } from 'react';
import { useMutation } from '@tanstack/react-query';
import { shellApi, SelfCheckResponse, TestResultEntry, QemuEvent } from '@/lib/api';
import { PlayCircle, CheckCircle, XCircle, Loader2, Clock, X } from 'lucide-react';

interface SelfCheckRunnerProps {
  disabled?: boolean;
  wsEvents?: QemuEvent[];
}

export function SelfCheckRunner({ disabled, wsEvents }: SelfCheckRunnerProps) {
  const [lastResult, setLastResult] = useState<SelfCheckResponse | null>(null);
  const [liveTests, setLiveTests] = useState<TestResultEntry[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const [wasCanceled, setWasCanceled] = useState(false);

  // Listen for WebSocket events
  useEffect(() => {
    if (!wsEvents || wsEvents.length === 0) return;

    const latestEvent = wsEvents[wsEvents.length - 1];

    switch (latestEvent.type) {
      case 'self_check_started':
        setIsRunning(true);
        setLiveTests([]);
        setWasCanceled(false);
        break;

      case 'self_check_test':
        setLiveTests((prev) => [
          ...prev,
          {
            name: latestEvent.name,
            passed: latestEvent.passed,
            timestamp: latestEvent.timestamp,
          },
        ]);
        break;

      case 'self_check_completed':
        setIsRunning(false);
        setWasCanceled(false);
        setLastResult((prev) => ({
          tests: prev?.tests || liveTests,
          total: latestEvent.total,
          passed: latestEvent.passed,
          failed: latestEvent.failed,
          success: latestEvent.success,
          execution_time_ms: prev?.execution_time_ms || 0,
        }));
        break;

      case 'self_check_canceled':
        setIsRunning(false);
        setWasCanceled(true);
        break;
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wsEvents]);

  const runSelfCheck = useMutation({
    mutationFn: () => shellApi.selfcheck(),
    onSuccess: (response) => {
      setLastResult(response);
      setIsRunning(false);
    },
    onError: () => {
      setIsRunning(false);
    },
  });

  const cancelSelfCheck = useMutation({
    mutationFn: () => shellApi.cancelSelfcheck(),
    onSuccess: () => {
      // State will be updated by WebSocket event
    },
    onError: () => {
      setIsRunning(false);
    },
  });

  const handleRun = () => {
    if (disabled) return;
    setIsRunning(true);
    setLiveTests([]);
    setWasCanceled(false);
    runSelfCheck.mutate();
  };

  const handleCancel = () => {
    cancelSelfCheck.mutate();
  };

  return (
    <div className="bg-card rounded-lg border p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Self-Check Tests</h3>
        <div className="flex gap-2">
          {isRunning ? (
            <button
              onClick={handleCancel}
              disabled={cancelSelfCheck.isPending}
              className="px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/90 disabled:opacity-50 flex items-center gap-2"
            >
              <X className="h-4 w-4" />
              {cancelSelfCheck.isPending ? 'Canceling...' : 'Cancel'}
            </button>
          ) : (
            <button
              onClick={handleRun}
              disabled={disabled}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 flex items-center gap-2"
            >
              <PlayCircle className="h-4 w-4" />
              Run Self-Check
            </button>
          )}
        </div>
      </div>

      {/* Live test results (streaming via WebSocket) */}
      {isRunning && liveTests.length > 0 && (
        <div className="space-y-2 mb-4">
          <h4 className="text-sm font-medium text-muted-foreground flex items-center gap-2">
            <Loader2 className="h-3 w-3 animate-spin" />
            Running Tests ({liveTests.length} completed)
          </h4>
          {liveTests.map((test, index) => (
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
      )}

      {/* Canceled message */}
      {wasCanceled && !isRunning && (
        <div className="mb-4 p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-md border border-yellow-200 dark:border-yellow-800">
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-yellow-600 dark:text-yellow-400" />
            <span className="font-semibold text-yellow-900 dark:text-yellow-100">
              Self-check canceled
            </span>
          </div>
          {liveTests.length > 0 && (
            <p className="text-sm text-yellow-800 dark:text-yellow-200 mt-2">
              {liveTests.length} test{liveTests.length !== 1 ? 's' : ''} completed before cancellation
            </p>
          )}
        </div>
      )}

      {/* Final results summary */}
      {lastResult && !isRunning && !wasCanceled && (
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
              {lastResult.execution_time_ms > 0 && (
                <span className="flex items-center gap-1 text-muted-foreground">
                  <Clock className="h-3 w-3" />
                  {lastResult.execution_time_ms}ms
                </span>
              )}
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

      {!lastResult && !isRunning && !wasCanceled && (
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
