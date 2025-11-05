/**
 * QEMU profile selector with feature flags
 */

import { useState, useEffect } from 'react';
import { Play, Square, Save, FolderOpen, Star, Trash2 } from 'lucide-react';
import { QemuConfig, QemuState } from '@/lib/api';
import {
  loadProfiles,
  saveProfile,
  deleteProfile,
  getDefaultProfile,
  setDefaultProfile,
  type QemuProfile,
} from '../lib/profiles';
import { showSuccessToast, showErrorToast } from '../lib/toast';

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
  const [profiles, setProfiles] = useState<QemuProfile[]>([]);
  const [defaultProfileName, setDefaultProfileName] = useState<string | null>(null);
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');

  // Load profiles and default on mount
  useEffect(() => {
    setProfiles(loadProfiles());
    setDefaultProfileName(getDefaultProfile());
  }, []);

  // Load default profile on mount if set
  useEffect(() => {
    if (defaultProfileName) {
      const defaultProfile = profiles.find((p) => p.name === defaultProfileName);
      if (defaultProfile) {
        loadProfileConfig(defaultProfile);
      }
    }
  }, [defaultProfileName]);

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

  const loadProfileConfig = (profile: QemuProfile) => {
    setSelectedFeatures(profile.features);
    // Custom env is not saved in profiles for now
    showSuccessToast(`Loaded profile: ${profile.name}`);
  };

  const handleSaveProfile = () => {
    if (!newProfileName.trim()) {
      showErrorToast('Profile name cannot be empty');
      return;
    }

    try {
      const profile: QemuProfile = {
        name: newProfileName.trim(),
        features: selectedFeatures,
        bringup: true,
      };
      saveProfile(profile);
      setProfiles(loadProfiles());
      setShowSaveDialog(false);
      setNewProfileName('');
      showSuccessToast(`Saved profile: ${profile.name}`);
    } catch (err) {
      showErrorToast('Failed to save profile');
    }
  };

  const handleDeleteProfile = (name: string) => {
    try {
      deleteProfile(name);
      setProfiles(loadProfiles());
      if (defaultProfileName === name) {
        setDefaultProfile(null);
        setDefaultProfileName(null);
      }
      showSuccessToast(`Deleted profile: ${name}`);
    } catch (err) {
      showErrorToast('Failed to delete profile');
    }
  };

  const handleSetDefault = (name: string) => {
    try {
      setDefaultProfile(name);
      setDefaultProfileName(name);
      showSuccessToast(`Set ${name} as default profile`);
    } catch (err) {
      showErrorToast('Failed to set default profile');
    }
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

        {/* Save Current Profile */}
        <div className="border-t pt-4">
          <div className="flex items-center justify-between mb-2">
            <label className="text-sm font-medium">Save Current Profile</label>
            {!showSaveDialog && (
              <button
                onClick={() => setShowSaveDialog(true)}
                disabled={isRunning}
                className="flex items-center gap-1 px-2 py-1 text-xs bg-muted hover:bg-muted/80 rounded disabled:opacity-50"
              >
                <Save className="h-3 w-3" />
                Save
              </button>
            )}
          </div>

          {showSaveDialog && (
            <div className="space-y-2 p-3 bg-muted/50 rounded-md">
              <input
                type="text"
                value={newProfileName}
                onChange={(e) => setNewProfileName(e.target.value)}
                placeholder="Profile name..."
                className="w-full px-3 py-2 text-sm border rounded-md bg-background"
                autoFocus
              />
              <div className="flex gap-2">
                <button
                  onClick={handleSaveProfile}
                  className="flex-1 px-3 py-1.5 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 text-sm font-medium"
                >
                  Save Profile
                </button>
                <button
                  onClick={() => {
                    setShowSaveDialog(false);
                    setNewProfileName('');
                  }}
                  className="px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md text-sm"
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Saved Profiles */}
        {profiles.length > 0 && (
          <div className="border-t pt-4">
            <label className="text-sm font-medium mb-2 block">
              Saved Profiles
            </label>
            <div className="space-y-2">
              {profiles.map((profile) => {
                const isDefault = defaultProfileName === profile.name;
                return (
                  <div
                    key={profile.name}
                    className={`flex items-center gap-2 p-2 rounded-md border ${
                      isDefault
                        ? 'bg-primary/10 border-primary'
                        : 'bg-muted/30 border-border'
                    }`}
                  >
                    <button
                      onClick={() => handleSetDefault(profile.name)}
                      disabled={isRunning}
                      className={`flex-shrink-0 p-1 rounded hover:bg-accent disabled:opacity-50 ${
                        isDefault ? 'text-primary' : 'text-muted-foreground'
                      }`}
                      title={isDefault ? 'Default profile' : 'Set as default'}
                    >
                      <Star
                        className={`h-4 w-4 ${isDefault ? 'fill-current' : ''}`}
                      />
                    </button>

                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium truncate">
                        {profile.name}
                      </p>
                      <p className="text-xs text-muted-foreground">
                        {profile.features.length} feature
                        {profile.features.length !== 1 ? 's' : ''}
                      </p>
                    </div>

                    <button
                      onClick={() => loadProfileConfig(profile)}
                      disabled={isRunning}
                      className="flex items-center gap-1 px-2 py-1 text-xs bg-background hover:bg-accent rounded disabled:opacity-50"
                      title="Load profile"
                    >
                      <FolderOpen className="h-3 w-3" />
                      Load
                    </button>

                    <button
                      onClick={() => handleDeleteProfile(profile.name)}
                      disabled={isRunning}
                      className="flex items-center gap-1 px-2 py-1 text-xs text-destructive hover:bg-destructive/10 rounded disabled:opacity-50"
                      title="Delete profile"
                    >
                      <Trash2 className="h-3 w-3" />
                    </button>
                  </div>
                );
              })}
            </div>
          </div>
        )}

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
