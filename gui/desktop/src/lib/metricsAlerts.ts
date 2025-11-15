/**
 * Performance alerts and threshold management - M5 Enhancement
 *
 * Provides threshold-based alerting for metrics with configurable rules.
 */

import { MetricPoint } from './api';

export interface ThresholdRule {
  id: string;
  metricName: string;
  operator: 'gt' | 'lt' | 'eq';
  threshold: number;
  severity: 'critical' | 'warning' | 'info';
  message: string;
  enabled: boolean;
}

export interface AlertEvent {
  id: string;
  ruleId: string;
  metricName: string;
  value: number;
  threshold: number;
  severity: 'critical' | 'warning' | 'info';
  message: string;
  timestamp: number;
}

const STORAGE_KEY = 'sis-metrics-thresholds';
const ALERT_HISTORY_KEY = 'sis-metrics-alerts';
const MAX_ALERT_HISTORY = 100;

class MetricsAlertManager {
  private rules: Map<string, ThresholdRule> = new Map();
  private alertHistory: AlertEvent[] = [];
  private listeners: Array<(alert: AlertEvent) => void> = [];

  constructor() {
    this.loadRules();
    this.loadAlertHistory();
  }

  loadRules() {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const rules = JSON.parse(stored) as ThresholdRule[];
        this.rules = new Map(rules.map((rule) => [rule.id, rule]));
      } else {
        // Load default rules
        this.loadDefaultRules();
      }
    } catch (error) {
      console.error('Failed to load threshold rules:', error);
      this.loadDefaultRules();
    }
  }

  loadDefaultRules() {
    const defaults: ThresholdRule[] = [
      {
        id: 'irq_latency_critical',
        metricName: 'irq_latency_ns',
        operator: 'gt',
        threshold: 10000,
        severity: 'critical',
        message: 'IRQ latency exceeds 10μs',
        enabled: true,
      },
      {
        id: 'irq_latency_warning',
        metricName: 'irq_latency_ns',
        operator: 'gt',
        threshold: 5000,
        severity: 'warning',
        message: 'IRQ latency exceeds 5μs',
        enabled: true,
      },
      {
        id: 'mem_low',
        metricName: 'mem_free_kb',
        operator: 'lt',
        threshold: 10240,
        severity: 'warning',
        message: 'Low memory: less than 10MB free',
        enabled: true,
      },
      {
        id: 'cpu_high',
        metricName: 'cpu_util',
        operator: 'gt',
        threshold: 0.9,
        severity: 'warning',
        message: 'High CPU utilization: above 90%',
        enabled: true,
      },
    ];

    defaults.forEach((rule) => this.rules.set(rule.id, rule));
    this.saveRules();
  }

  loadAlertHistory() {
    try {
      const stored = localStorage.getItem(ALERT_HISTORY_KEY);
      if (stored) {
        this.alertHistory = JSON.parse(stored);
      }
    } catch (error) {
      console.error('Failed to load alert history:', error);
      this.alertHistory = [];
    }
  }

  saveRules() {
    try {
      const rules = Array.from(this.rules.values());
      localStorage.setItem(STORAGE_KEY, JSON.stringify(rules));
    } catch (error) {
      console.error('Failed to save threshold rules:', error);
    }
  }

  saveAlertHistory() {
    try {
      // Keep only recent alerts
      const recent = this.alertHistory.slice(0, MAX_ALERT_HISTORY);
      localStorage.setItem(ALERT_HISTORY_KEY, JSON.stringify(recent));
    } catch (error) {
      console.error('Failed to save alert history:', error);
    }
  }

  addRule(rule: ThresholdRule) {
    this.rules.set(rule.id, rule);
    this.saveRules();
  }

  updateRule(id: string, updates: Partial<ThresholdRule>) {
    const rule = this.rules.get(id);
    if (rule) {
      this.rules.set(id, { ...rule, ...updates });
      this.saveRules();
    }
  }

  deleteRule(id: string) {
    this.rules.delete(id);
    this.saveRules();
  }

  getRules(): ThresholdRule[] {
    return Array.from(this.rules.values());
  }

  getRule(id: string): ThresholdRule | undefined {
    return this.rules.get(id);
  }

  getRulesForMetric(metricName: string): ThresholdRule[] {
    return this.getRules().filter((rule) => rule.metricName === metricName);
  }

  checkPoint(metricName: string, point: MetricPoint): AlertEvent[] {
    const rules = this.getRulesForMetric(metricName).filter((r) => r.enabled);
    const alerts: AlertEvent[] = [];

    for (const rule of rules) {
      let triggered = false;

      switch (rule.operator) {
        case 'gt':
          triggered = point.value > rule.threshold;
          break;
        case 'lt':
          triggered = point.value < rule.threshold;
          break;
        case 'eq':
          triggered = Math.abs(point.value - rule.threshold) < 0.001;
          break;
      }

      if (triggered) {
        const alert: AlertEvent = {
          id: `${rule.id}-${point.ts}`,
          ruleId: rule.id,
          metricName,
          value: point.value,
          threshold: rule.threshold,
          severity: rule.severity,
          message: rule.message,
          timestamp: point.ts,
        };

        alerts.push(alert);
        this.recordAlert(alert);
      }
    }

    return alerts;
  }

  private recordAlert(alert: AlertEvent) {
    // Add to history
    this.alertHistory.unshift(alert);

    // Trim history
    if (this.alertHistory.length > MAX_ALERT_HISTORY) {
      this.alertHistory = this.alertHistory.slice(0, MAX_ALERT_HISTORY);
    }

    this.saveAlertHistory();

    // Notify listeners
    this.listeners.forEach((listener) => listener(alert));
  }

  onAlert(listener: (alert: AlertEvent) => void) {
    this.listeners.push(listener);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  getAlertHistory(limit?: number): AlertEvent[] {
    return limit ? this.alertHistory.slice(0, limit) : this.alertHistory;
  }

  clearAlertHistory() {
    this.alertHistory = [];
    this.saveAlertHistory();
  }
}

export const metricsAlertManager = new MetricsAlertManager();
