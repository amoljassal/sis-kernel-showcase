/**
 * Settings Panel - M6
 *
 * Application settings including:
 * - Theme selection (light/dark/system)
 * - Keyboard shortcuts
 * - Daemon configuration
 * - General preferences
 */

import { useState, useEffect } from 'react';
import { Settings, Monitor, Moon, Sun, Keyboard, Server, Save, RotateCcw } from 'lucide-react';
import { showToast } from '../lib/toast';

type Theme = 'light' | 'dark' | 'system';

interface AppSettings {
  theme: Theme;
  shortcuts: Record<string, string>;
  daemonUrl: string;
  wsUrl: string;
  autoReconnect: boolean;
  reconnectInterval: number;
}

const DEFAULT_SETTINGS: AppSettings = {
  theme: 'system',
  shortcuts: {
    'toggle-qemu': 'Ctrl+Q',
    'open-terminal': 'Ctrl+T',
    'open-metrics': 'Ctrl+M',
    'open-logs': 'Ctrl+L',
    'open-settings': 'Ctrl+,',
    'save-profile': 'Ctrl+S',
  },
  daemonUrl: 'http://localhost:8871',
  wsUrl: 'ws://localhost:8871/events',
  autoReconnect: true,
  reconnectInterval: 5000,
};

const STORAGE_KEY = 'sis-app-settings';

export function SettingsPanel() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [hasChanges, setHasChanges] = useState(false);
  const [activeTab, setActiveTab] = useState<'appearance' | 'shortcuts' | 'daemon'>('appearance');

  // Load settings from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        setSettings({ ...DEFAULT_SETTINGS, ...parsed });
      }
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  }, []);

  // Apply theme
  useEffect(() => {
    applyTheme(settings.theme);
  }, [settings.theme]);

  const applyTheme = (theme: Theme) => {
    const root = document.documentElement;

    if (theme === 'system') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', prefersDark);
    } else {
      root.classList.toggle('dark', theme === 'dark');
    }
  };

  const handleSave = () => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
      setHasChanges(false);
      showToast('Settings saved successfully', 'success');

      // Apply changes
      applyTheme(settings.theme);
    } catch (error) {
      console.error('Failed to save settings:', error);
      showToast('Failed to save settings', 'error');
    }
  };

  const handleReset = () => {
    if (confirm('Reset all settings to defaults?')) {
      setSettings(DEFAULT_SETTINGS);
      setHasChanges(true);
      showToast('Settings reset to defaults', 'info');
    }
  };

  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setHasChanges(true);
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="border-b p-4 flex items-center justify-between">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Settings className="h-6 w-6 text-primary" />
          Settings
        </h2>
        <div className="flex items-center gap-2">
          {hasChanges && (
            <span className="text-sm text-orange-600 dark:text-orange-400 font-medium">
              Unsaved changes
            </span>
          )}
          <button
            onClick={handleReset}
            className="px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md text-sm flex items-center gap-2"
          >
            <RotateCcw className="h-4 w-4" />
            Reset
          </button>
          <button
            onClick={handleSave}
            disabled={!hasChanges}
            className="px-4 py-1.5 bg-primary text-primary-foreground hover:bg-primary/90 rounded-md text-sm flex items-center gap-2 disabled:opacity-50"
          >
            <Save className="h-4 w-4" />
            Save Changes
          </button>
        </div>
      </div>

      <div className="flex-1 overflow-hidden flex">
        {/* Sidebar tabs */}
        <div className="w-48 border-r p-2 space-y-1">
          <button
            onClick={() => setActiveTab('appearance')}
            className={`w-full text-left px-3 py-2 rounded-md flex items-center gap-2 transition-colors ${
              activeTab === 'appearance'
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted'
            }`}
          >
            <Sun className="h-4 w-4" />
            Appearance
          </button>
          <button
            onClick={() => setActiveTab('shortcuts')}
            className={`w-full text-left px-3 py-2 rounded-md flex items-center gap-2 transition-colors ${
              activeTab === 'shortcuts'
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted'
            }`}
          >
            <Keyboard className="h-4 w-4" />
            Shortcuts
          </button>
          <button
            onClick={() => setActiveTab('daemon')}
            className={`w-full text-left px-3 py-2 rounded-md flex items-center gap-2 transition-colors ${
              activeTab === 'daemon' ? 'bg-primary text-primary-foreground' : 'hover:bg-muted'
            }`}
          >
            <Server className="h-4 w-4" />
            Daemon
          </button>
        </div>

        {/* Content area */}
        <div className="flex-1 overflow-y-auto p-6">
          {activeTab === 'appearance' && (
            <div className="space-y-6 max-w-2xl">
              <div>
                <h3 className="text-lg font-semibold mb-4">Appearance</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Customize how the application looks
                </p>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-3">Theme</label>
                  <div className="grid grid-cols-3 gap-3">
                    <button
                      onClick={() => updateSetting('theme', 'light')}
                      className={`p-4 rounded-lg border-2 transition-all ${
                        settings.theme === 'light'
                          ? 'border-primary bg-primary/5'
                          : 'border-border hover:border-primary/50'
                      }`}
                    >
                      <Sun className="h-6 w-6 mx-auto mb-2 text-yellow-600" />
                      <div className="text-sm font-medium">Light</div>
                    </button>

                    <button
                      onClick={() => updateSetting('theme', 'dark')}
                      className={`p-4 rounded-lg border-2 transition-all ${
                        settings.theme === 'dark'
                          ? 'border-primary bg-primary/5'
                          : 'border-border hover:border-primary/50'
                      }`}
                    >
                      <Moon className="h-6 w-6 mx-auto mb-2 text-blue-600" />
                      <div className="text-sm font-medium">Dark</div>
                    </button>

                    <button
                      onClick={() => updateSetting('theme', 'system')}
                      className={`p-4 rounded-lg border-2 transition-all ${
                        settings.theme === 'system'
                          ? 'border-primary bg-primary/5'
                          : 'border-border hover:border-primary/50'
                      }`}
                    >
                      <Monitor className="h-6 w-6 mx-auto mb-2 text-purple-600" />
                      <div className="text-sm font-medium">System</div>
                    </button>
                  </div>
                  <p className="text-xs text-muted-foreground mt-2">
                    System theme follows your operating system preference
                  </p>
                </div>
              </div>
            </div>
          )}

          {activeTab === 'shortcuts' && (
            <div className="space-y-6 max-w-2xl">
              <div>
                <h3 className="text-lg font-semibold mb-4">Keyboard Shortcuts</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Customize keyboard shortcuts for common actions
                </p>
              </div>

              <div className="space-y-3">
                {Object.entries(settings.shortcuts).map(([action, shortcut]) => (
                  <div
                    key={action}
                    className="flex items-center justify-between p-3 bg-muted rounded-md"
                  >
                    <div>
                      <div className="font-medium text-sm capitalize">
                        {action.replace(/-/g, ' ')}
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Keyboard shortcut for this action
                      </div>
                    </div>
                    <kbd className="px-3 py-1.5 bg-background border rounded font-mono text-sm">
                      {shortcut}
                    </kbd>
                  </div>
                ))}
              </div>

              <div className="bg-blue-500/10 border border-blue-500/20 rounded-md p-3 text-sm">
                <p className="text-blue-900 dark:text-blue-100">
                  <strong>Note:</strong> Keyboard shortcut customization is coming soon.
                  Currently showing default shortcuts.
                </p>
              </div>
            </div>
          )}

          {activeTab === 'daemon' && (
            <div className="space-y-6 max-w-2xl">
              <div>
                <h3 className="text-lg font-semibold mb-4">Daemon Configuration</h3>
                <p className="text-sm text-muted-foreground mb-4">
                  Configure connection settings for the sisctl daemon
                </p>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-2">Daemon REST URL</label>
                  <input
                    type="text"
                    value={settings.daemonUrl}
                    onChange={(e) => updateSetting('daemonUrl', e.target.value)}
                    className="w-full px-3 py-2 bg-background border rounded-md font-mono text-sm"
                    placeholder="http://localhost:8871"
                  />
                  <p className="text-xs text-muted-foreground mt-1">
                    Base URL for REST API endpoints
                  </p>
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">WebSocket URL</label>
                  <input
                    type="text"
                    value={settings.wsUrl}
                    onChange={(e) => updateSetting('wsUrl', e.target.value)}
                    className="w-full px-3 py-2 bg-background border rounded-md font-mono text-sm"
                    placeholder="ws://localhost:8871/events"
                  />
                  <p className="text-xs text-muted-foreground mt-1">
                    WebSocket endpoint for real-time events
                  </p>
                </div>

                <div className="pt-4 border-t">
                  <label className="flex items-center gap-3 cursor-pointer">
                    <input
                      type="checkbox"
                      checked={settings.autoReconnect}
                      onChange={(e) => updateSetting('autoReconnect', e.target.checked)}
                      className="w-4 h-4 rounded border-gray-300"
                    />
                    <div>
                      <div className="font-medium text-sm">Auto-reconnect</div>
                      <div className="text-xs text-muted-foreground">
                        Automatically reconnect to daemon if connection is lost
                      </div>
                    </div>
                  </label>
                </div>

                {settings.autoReconnect && (
                  <div>
                    <label className="block text-sm font-medium mb-2">
                      Reconnect Interval (ms)
                    </label>
                    <input
                      type="number"
                      value={settings.reconnectInterval}
                      onChange={(e) =>
                        updateSetting('reconnectInterval', parseInt(e.target.value, 10))
                      }
                      min="1000"
                      max="60000"
                      step="1000"
                      className="w-full px-3 py-2 bg-background border rounded-md"
                    />
                    <p className="text-xs text-muted-foreground mt-1">
                      How often to attempt reconnection (1000-60000ms)
                    </p>
                  </div>
                )}
              </div>

              <div className="bg-orange-500/10 border border-orange-500/20 rounded-md p-3 text-sm">
                <p className="text-orange-900 dark:text-orange-100">
                  <strong>Warning:</strong> Changing daemon URLs requires a page reload to take
                  effect. Make sure the daemon is running at the specified address.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
