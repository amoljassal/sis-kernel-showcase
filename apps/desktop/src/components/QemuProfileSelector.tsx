/**
 * QEMU profile selector with feature flags
 */

import { useState } from 'react';
import { Play, Square } from 'lucide-react';
import { QemuConfig, QemuState } from '@/lib/api';

interface QemuProfileSelectorProps {
  onRun: (config: QemuConfig) => void;
  onStop: () => void;
  qemuState: QemuState;
}

const AVAILABLE_FEATURES = [
  { id: 'llm', label: 'LLM Support', description: 'Enable LLM subsystem' },
  {
    id: 'graph-demo',
    label: 'Graph Demo',
    description: 'Include graph computation demos',
  },
  {
    id: 'perf-verbose',
    label: 'Verbose Performance',
    description: 'Detailed performance metrics',
  },
  {
    id: 'virtio-console',
    label: 'VirtIO Console',
    description: 'Enable VirtIO console driver',
  },
  {
    id: 'deterministic',
    label: 'Deterministic Scheduler',
    description: 'CBS+EDF scheduling',
  },
  { id: 'demos', label: 'All Demos', description: 'Enable all demo commands' },
];

export function QemuProfileSelector({
  onRun,
  onStop,
  qemuState,
}: QemuProfileSelectorProps) {
  const [selectedFeatures, setSelectedFeatures] = useState<string[]>([]);
  const [customEnv, setCustomEnv] = useState('');

  const isRunning =
    qemuState === QemuState.Running || qemuState === QemuState.Starting;
  const isStopping = qemuState === QemuState.Stopping;

  const toggleFeature = (featureId: string) => {
    setSelectedFeatures((prev) =>
      prev.includes(featureId)
        ? prev.filter((f) => f !== featureId)
        : [...prev, featureId]
    );
  };

  const handleRun = () => {
    const env: Record<string, string> = {
      BRINGUP: '1',
    };

    // Parse custom env (format: KEY=VALUE)
    if (customEnv.trim()) {
      customEnv.split('\n').forEach((line) => {
        const [key, value] = line.split('=');
        if (key && value) {
          env[key.trim()] = value.trim();
        }
      });
    }

    onRun({
      features: selectedFeatures,
      env,
    });
  };

  return (
    <div className="bg-card rounded-lg border p-4">
      <h3 className="text-lg font-semibold mb-4">QEMU Profile</h3>

      <div className="space-y-4">
        {/* Feature Selection */}
        <div>
          <label className="text-sm font-medium mb-2 block">
            Feature Flags
          </label>
          <div className="space-y-2">
            {AVAILABLE_FEATURES.map((feature) => (
              <label
                key={feature.id}
                className="flex items-start gap-3 p-2 rounded hover:bg-accent cursor-pointer"
              >
                <input
                  type="checkbox"
                  checked={selectedFeatures.includes(feature.id)}
                  onChange={() => toggleFeature(feature.id)}
                  disabled={isRunning}
                  className="mt-1"
                />
                <div className="flex-1">
                  <p className="text-sm font-medium">{feature.label}</p>
                  <p className="text-xs text-muted-foreground">
                    {feature.description}
                  </p>
                </div>
              </label>
            ))}
          </div>
        </div>

        {/* Custom Environment */}
        <div>
          <label className="text-sm font-medium mb-2 block">
            Custom Environment
          </label>
          <textarea
            value={customEnv}
            onChange={(e) => setCustomEnv(e.target.value)}
            disabled={isRunning}
            placeholder="KEY=VALUE (one per line)"
            className="w-full h-20 px-3 py-2 text-sm border rounded-md bg-background resize-none"
          />
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          {!isRunning ? (
            <button
              onClick={handleRun}
              disabled={isStopping}
              className="flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50"
            >
              <Play className="h-4 w-4" />
              Run QEMU
            </button>
          ) : (
            <button
              onClick={onStop}
              className="flex items-center gap-2 px-4 py-2 bg-destructive text-destructive-foreground rounded-md hover:bg-destructive/90"
            >
              <Square className="h-4 w-4" />
              Stop QEMU
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
