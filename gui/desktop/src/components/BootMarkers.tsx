/**
 * Boot markers checklist component
 */

import { Check, Loader2 } from 'lucide-react';

interface BootMarkersProps {
  markers: Record<string, boolean>;
}

const BOOT_MARKERS = [
  { key: 'KernelU', label: 'Kernel entry point', description: 'KERNEL(U)' },
  { key: 'StackOk', label: 'Stack initialized', description: 'STACK OK' },
  { key: 'MmuSctlr', label: 'MMU control register', description: 'MMU: SCTLR' },
  { key: 'MmuOn', label: 'MMU enabled', description: 'MMU ON' },
  { key: 'UartReady', label: 'UART driver ready', description: 'UART: READY' },
  { key: 'GicInit', label: 'GIC initialized', description: 'GIC: INIT' },
  { key: 'VectorsOk', label: 'Exception vectors', description: 'VECTORS OK' },
  { key: 'LaunchingShell', label: 'Shell launching', description: 'LAUNCHING SHELL' },
  { key: 'ShellReady', label: 'Shell ready', description: 'sis> prompt' },
];

export function BootMarkers({ markers }: BootMarkersProps) {
  const completedCount = Object.values(markers).filter(Boolean).length;
  const totalCount = BOOT_MARKERS.length;
  const isComplete = completedCount === totalCount;

  return (
    <div className="bg-card rounded-lg border p-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">Boot Progress</h3>
        <span className="text-sm text-muted-foreground">
          {completedCount} / {totalCount}
        </span>
      </div>

      <div className="space-y-2">
        {BOOT_MARKERS.map(({ key, label, description }) => {
          const isComplete = markers[key] || false;
          const isNext =
            !isComplete &&
            BOOT_MARKERS.findIndex((m) => !markers[m.key]) ===
              BOOT_MARKERS.findIndex((m) => m.key === key);

          return (
            <div
              key={key}
              className={`flex items-start gap-3 p-2 rounded ${
                isComplete
                  ? 'bg-green-50 dark:bg-green-900/20'
                  : isNext
                  ? 'bg-yellow-50 dark:bg-yellow-900/20'
                  : 'bg-muted/50'
              }`}
            >
              <div className="mt-0.5">
                {isComplete ? (
                  <Check className="h-5 w-5 text-green-600 dark:text-green-400" />
                ) : isNext ? (
                  <Loader2 className="h-5 w-5 text-yellow-600 dark:text-yellow-400 animate-spin" />
                ) : (
                  <div className="h-5 w-5 rounded-full border-2 border-muted" />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <p
                  className={`text-sm font-medium ${
                    isComplete
                      ? 'text-green-900 dark:text-green-100'
                      : 'text-foreground'
                  }`}
                >
                  {label}
                </p>
                <p className="text-xs text-muted-foreground">{description}</p>
              </div>
            </div>
          );
        })}
      </div>

      {isComplete && (
        <div className="mt-4 p-3 bg-green-50 dark:bg-green-900/20 rounded border border-green-200 dark:border-green-800">
          <p className="text-sm font-medium text-green-900 dark:text-green-100">
            ✓ BOOT_COMPLETE — Boot sequence complete!
          </p>
        </div>
      )}
    </div>
  );
}
