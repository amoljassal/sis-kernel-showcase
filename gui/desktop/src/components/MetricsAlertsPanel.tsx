/**
 * Metrics Alerts Panel - M5 Enhancement
 *
 * Displays performance alerts based on threshold rules.
 * Allows configuration of alert thresholds.
 */

import { useState, useEffect } from 'react';
import { Bell, BellOff, Plus, Trash2, AlertTriangle, AlertCircle, Info, Settings } from 'lucide-react';
import {
  metricsAlertManager,
  type ThresholdRule,
  type AlertEvent,
} from '../lib/metricsAlerts';
import { showToast } from '../lib/toast';

export function MetricsAlertsPanel() {
  const [alerts, setAlerts] = useState<AlertEvent[]>([]);
  const [rules, setRules] = useState<ThresholdRule[]>([]);
  // Reserved for future rule editor feature
  const [_showRuleEditor, _setShowRuleEditor] = useState(false);
  const [_editingRule, _setEditingRule] = useState<ThresholdRule | null>(null);

  // Load rules and alerts
  useEffect(() => {
    setRules(metricsAlertManager.getRules());
    setAlerts(metricsAlertManager.getAlertHistory(50));

    // Subscribe to new alerts
    const unsubscribe = metricsAlertManager.onAlert((alert) => {
      setAlerts((prev) => [alert, ...prev].slice(0, 50));

      // Show toast for critical alerts
      if (alert.severity === 'critical') {
        showToast(`${alert.message}: ${alert.value.toFixed(2)}`, 'error');
      } else if (alert.severity === 'warning') {
        showToast(`${alert.message}: ${alert.value.toFixed(2)}`, 'error');
      }
    });

    return unsubscribe;
  }, []);

  const toggleRule = (id: string) => {
    const rule = rules.find((r) => r.id === id);
    if (rule) {
      metricsAlertManager.updateRule(id, { enabled: !rule.enabled });
      setRules(metricsAlertManager.getRules());
      showToast(`Alert rule ${rule.enabled ? 'disabled' : 'enabled'}`, 'info');
    }
  };

  const deleteRule = (id: string) => {
    if (confirm('Delete this alert rule?')) {
      metricsAlertManager.deleteRule(id);
      setRules(metricsAlertManager.getRules());
      showToast('Alert rule deleted', 'info');
    }
  };

  const clearHistory = () => {
    if (confirm('Clear all alert history?')) {
      metricsAlertManager.clearAlertHistory();
      setAlerts([]);
      showToast('Alert history cleared', 'success');
    }
  };

  const getSeverityIcon = (severity: string) => {
    switch (severity) {
      case 'critical':
        return <AlertCircle className="h-5 w-5 text-red-600" />;
      case 'warning':
        return <AlertTriangle className="h-5 w-5 text-yellow-600" />;
      default:
        return <Info className="h-5 w-5 text-blue-600" />;
    }
  };

  const getSeverityClass = (severity: string) => {
    switch (severity) {
      case 'critical':
        return 'bg-red-500/10 border-red-500/20 text-red-900 dark:text-red-100';
      case 'warning':
        return 'bg-yellow-500/10 border-yellow-500/20 text-yellow-900 dark:text-yellow-100';
      default:
        return 'bg-blue-500/10 border-blue-500/20 text-blue-900 dark:text-blue-100';
    }
  };

  const formatValue = (value: number) => {
    if (value > 1000000) return `${(value / 1000000).toFixed(2)}M`;
    if (value > 1000) return `${(value / 1000).toFixed(2)}K`;
    return value.toFixed(2);
  };

  const getOperatorSymbol = (operator: string) => {
    switch (operator) {
      case 'gt':
        return '>';
      case 'lt':
        return '<';
      case 'eq':
        return '=';
      default:
        return operator;
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border p-4 gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-semibold flex items-center gap-2">
          <Bell className="h-6 w-6 text-primary" />
          Performance Alerts
        </h2>
        <div className="flex items-center gap-2">
          <span className="text-sm text-muted-foreground">
            {alerts.length} alerts
          </span>
          <button
            onClick={clearHistory}
            className="px-3 py-1.5 bg-muted hover:bg-muted/80 rounded-md text-sm flex items-center gap-2"
            disabled={alerts.length === 0}
          >
            <Trash2 className="h-4 w-4" />
            Clear
          </button>
        </div>
      </div>

      <div className="flex-1 grid grid-cols-2 gap-4 overflow-hidden">
        {/* Alert Rules */}
        <div className="flex flex-col gap-4">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Alert Rules ({rules.length})
            </h3>
          </div>

          <div className="flex-1 overflow-y-auto space-y-2">
            {rules.length === 0 ? (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                <p className="text-sm">No alert rules configured</p>
              </div>
            ) : (
              rules.map((rule) => (
                <div
                  key={rule.id}
                  className={`p-3 rounded-md border transition-all ${
                    rule.enabled ? 'bg-muted/50' : 'bg-muted/20 opacity-50'
                  }`}
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2">
                      {getSeverityIcon(rule.severity)}
                      <span className="font-medium text-sm">{rule.message}</span>
                    </div>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => toggleRule(rule.id)}
                        className="p-1 hover:bg-muted rounded transition-colors"
                        title={rule.enabled ? 'Disable' : 'Enable'}
                      >
                        {rule.enabled ? (
                          <Bell className="h-4 w-4" />
                        ) : (
                          <BellOff className="h-4 w-4" />
                        )}
                      </button>
                      <button
                        onClick={() => deleteRule(rule.id)}
                        className="p-1 hover:bg-destructive/10 text-destructive rounded transition-colors"
                        title="Delete"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </div>

                  <div className="text-xs font-mono text-muted-foreground space-y-1">
                    <div>Metric: {rule.metricName}</div>
                    <div>
                      Condition: value {getOperatorSymbol(rule.operator)}{' '}
                      {formatValue(rule.threshold)}
                    </div>
                    <div
                      className={`inline-block px-2 py-0.5 rounded text-xs font-semibold ${
                        rule.severity === 'critical'
                          ? 'bg-red-500/20 text-red-700'
                          : rule.severity === 'warning'
                          ? 'bg-yellow-500/20 text-yellow-700'
                          : 'bg-blue-500/20 text-blue-700'
                      }`}
                    >
                      {rule.severity}
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Alert History */}
        <div className="flex flex-col gap-4">
          <h3 className="font-semibold flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            Recent Alerts
          </h3>

          <div className="flex-1 overflow-y-auto space-y-2">
            {alerts.length === 0 ? (
              <div className="flex items-center justify-center h-full text-muted-foreground">
                <div className="text-center">
                  <Bell className="h-12 w-12 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">No alerts triggered</p>
                  <p className="text-xs mt-1">System is operating normally</p>
                </div>
              </div>
            ) : (
              alerts.map((alert) => (
                <div
                  key={alert.id}
                  className={`p-3 rounded-md border ${getSeverityClass(alert.severity)}`}
                >
                  <div className="flex items-start gap-2 mb-2">
                    {getSeverityIcon(alert.severity)}
                    <div className="flex-1">
                      <p className="font-medium text-sm">{alert.message}</p>
                      <p className="text-xs text-muted-foreground mt-1">
                        {new Date(alert.timestamp).toLocaleString()}
                      </p>
                    </div>
                  </div>

                  <div className="bg-background/50 rounded px-2 py-1 font-mono text-xs">
                    <span className="text-muted-foreground">{alert.metricName}:</span>{' '}
                    <span className="font-semibold">{formatValue(alert.value)}</span>
                    <span className="text-muted-foreground">
                      {' '}
                      (threshold: {formatValue(alert.threshold)})
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Info banner */}
      <div className="bg-muted rounded-md p-3 text-sm text-muted-foreground">
        <Info className="h-4 w-4 inline-block mr-2" />
        Alert rules are checked against incoming metrics in real-time. Configure thresholds
        based on your performance requirements. IndexedDB persistence ensures alerts survive
        page reloads.
      </div>
    </div>
  );
}
