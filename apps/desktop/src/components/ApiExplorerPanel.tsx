/**
 * API Explorer Panel - M3
 *
 * Features:
 * - Enhanced Swagger UI integration
 * - Request history with localStorage persistence
 * - Custom request builder
 * - Endpoint search and filtering
 * - Response formatting and inspection
 */

import { useState, useEffect, useMemo } from 'react';
import { Search, Clock, Send, Copy, Trash2, ExternalLink, ChevronDown, ChevronRight } from 'lucide-react';
import { copyToClipboard } from '../lib/clipboard';
import { showToast } from '../lib/toast';
import axios, { type Method } from 'axios';

// API endpoint metadata (derived from OpenAPI schema)
interface ApiEndpoint {
  method: Method;
  path: string;
  tag: string;
  summary?: string;
  description?: string;
}

// Request history entry
interface RequestHistoryEntry {
  id: string;
  timestamp: number;
  method: Method;
  url: string;
  headers?: Record<string, string>;
  body?: any;
  status?: number;
  responseBody?: any;
  error?: string;
  duration?: number;
}

// All available endpoints from OpenAPI schema
const API_ENDPOINTS: ApiEndpoint[] = [
  { method: 'GET', path: '/health', tag: 'health', summary: 'Health check' },
  { method: 'GET', path: '/api/v1/config', tag: 'config', summary: 'Get daemon configuration' },
  { method: 'POST', path: '/api/v1/qemu/run', tag: 'qemu', summary: 'Start QEMU' },
  { method: 'POST', path: '/api/v1/qemu/stop', tag: 'qemu', summary: 'Stop QEMU' },
  { method: 'GET', path: '/api/v1/qemu/status', tag: 'qemu', summary: 'Get QEMU status' },
  { method: 'POST', path: '/api/v1/shell/exec', tag: 'shell', summary: 'Execute shell command' },
  { method: 'POST', path: '/api/v1/shell/selfcheck', tag: 'shell', summary: 'Run self-check' },
  { method: 'POST', path: '/api/v1/shell/selfcheck/cancel', tag: 'shell', summary: 'Cancel self-check' },
  { method: 'POST', path: '/api/v1/replay', tag: 'replay', summary: 'Start replay' },
  { method: 'POST', path: '/api/v1/replay/stop', tag: 'replay', summary: 'Stop replay' },
  { method: 'GET', path: '/api/v1/replay/status', tag: 'replay', summary: 'Get replay status' },
  { method: 'GET', path: '/api/v1/metrics/streams', tag: 'metrics', summary: 'List metric streams' },
  { method: 'GET', path: '/api/v1/metrics/query', tag: 'metrics', summary: 'Query time series' },
  { method: 'POST', path: '/api/v1/autonomy/on', tag: 'autonomy', summary: 'Enable autonomy' },
  { method: 'POST', path: '/api/v1/autonomy/off', tag: 'autonomy', summary: 'Disable autonomy' },
  { method: 'POST', path: '/api/v1/autonomy/reset', tag: 'autonomy', summary: 'Reset autonomy state' },
  { method: 'GET', path: '/api/v1/autonomy/status', tag: 'autonomy', summary: 'Get autonomy status' },
  { method: 'GET', path: '/api/v1/autonomy/audit', tag: 'autonomy', summary: 'Get audit trail' },
  { method: 'GET', path: '/api/v1/autonomy/explain', tag: 'autonomy', summary: 'Explain decision' },
  { method: 'POST', path: '/api/v1/autonomy/preview', tag: 'autonomy', summary: 'Preview decision' },
  { method: 'POST', path: '/api/v1/autonomy/whatif', tag: 'autonomy', summary: 'Run what-if simulation' },
  { method: 'GET', path: '/api/v1/mem/approvals', tag: 'memory', summary: 'Get memory approvals' },
  { method: 'POST', path: '/api/v1/mem/approval', tag: 'memory', summary: 'Toggle approval mode' },
  { method: 'POST', path: '/api/v1/mem/approve', tag: 'memory', summary: 'Approve operation' },
  { method: 'POST', path: '/api/v1/mem/reject', tag: 'memory', summary: 'Reject operation' },
  { method: 'POST', path: '/api/v1/graph/create', tag: 'graph', summary: 'Create graph' },
  { method: 'POST', path: '/api/v1/graph/add-channel', tag: 'graph', summary: 'Add channel' },
  { method: 'POST', path: '/api/v1/graph/add-operator', tag: 'graph', summary: 'Add operator' },
  { method: 'POST', path: '/api/v1/graph/start', tag: 'graph', summary: 'Start graph' },
  { method: 'POST', path: '/api/v1/graph/predict', tag: 'graph', summary: 'Make prediction' },
  { method: 'POST', path: '/api/v1/graph/feedback', tag: 'graph', summary: 'Provide feedback' },
  { method: 'GET', path: '/api/v1/graph/state', tag: 'graph', summary: 'Get graph state' },
  { method: 'POST', path: '/api/v1/graph/export', tag: 'graph', summary: 'Export graph' },
  { method: 'GET', path: '/api/v1/sched/workloads', tag: 'scheduling', summary: 'List workloads' },
  { method: 'POST', path: '/api/v1/sched/priorities', tag: 'scheduling', summary: 'Set priority' },
  { method: 'POST', path: '/api/v1/sched/affinity', tag: 'scheduling', summary: 'Set affinity' },
  { method: 'POST', path: '/api/v1/sched/feature', tag: 'scheduling', summary: 'Toggle feature' },
  { method: 'GET', path: '/api/v1/sched/circuit-breaker', tag: 'scheduling', summary: 'Get circuit breaker status' },
  { method: 'POST', path: '/api/v1/sched/circuit-breaker/reset', tag: 'scheduling', summary: 'Reset circuit breaker' },
  { method: 'POST', path: '/api/v1/llm/load', tag: 'llm', summary: 'Load LLM model' },
  { method: 'POST', path: '/api/v1/llm/infer', tag: 'llm', summary: 'Run inference' },
  { method: 'GET', path: '/api/v1/llm/audit', tag: 'llm', summary: 'Get audit trail' },
  { method: 'GET', path: '/api/v1/llm/status', tag: 'llm', summary: 'Get LLM status' },
  { method: 'GET', path: '/api/v1/logs/tail', tag: 'logs', summary: 'Tail logs' },
  { method: 'POST', path: '/api/v1/runs/start', tag: 'runs', summary: 'Start run' },
  { method: 'POST', path: '/api/v1/runs/stop', tag: 'runs', summary: 'Stop run' },
  { method: 'GET', path: '/api/v1/runs/list', tag: 'runs', summary: 'List runs' },
  { method: 'GET', path: '/api/v1/runs/:runId/export', tag: 'runs', summary: 'Export run logs' },
  { method: 'POST', path: '/api/v1/crash', tag: 'crashes', summary: 'Ingest crash' },
  { method: 'GET', path: '/api/v1/crashes', tag: 'crashes', summary: 'List crashes' },
  { method: 'POST', path: '/api/v1/incidents', tag: 'incidents', summary: 'Create incident' },
  { method: 'GET', path: '/api/v1/incidents', tag: 'incidents', summary: 'List incidents' },
];

const METHOD_COLORS: Record<Method, string> = {
  GET: 'bg-blue-500',
  POST: 'bg-green-500',
  PUT: 'bg-yellow-500',
  PATCH: 'bg-orange-500',
  DELETE: 'bg-red-500',
  HEAD: 'bg-purple-500',
  OPTIONS: 'bg-gray-500',
};

const STORAGE_KEY = 'sis-api-explorer-history';
const MAX_HISTORY_ENTRIES = 100;

export function ApiExplorerPanel() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [history, setHistory] = useState<RequestHistoryEntry[]>([]);
  const [selectedEndpoint, setSelectedEndpoint] = useState<ApiEndpoint | null>(null);
  const [customMethod, setCustomMethod] = useState<Method>('GET');
  const [customUrl, setCustomUrl] = useState('');
  const [customHeaders, setCustomHeaders] = useState('{}');
  const [customBody, setCustomBody] = useState('{}');
  const [response, setResponse] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [expandedTags, setExpandedTags] = useState<Set<string>>(new Set(['qemu', 'replay']));
  const [viewMode, setViewMode] = useState<'explorer' | 'history' | 'swagger'>('explorer');

  // Load history from localStorage
  useEffect(() => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        setHistory(JSON.parse(stored));
      }
    } catch (error) {
      console.error('Failed to load request history:', error);
    }
  }, []);

  // Save history to localStorage
  const saveHistory = (newHistory: RequestHistoryEntry[]) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(newHistory.slice(0, MAX_HISTORY_ENTRIES)));
      setHistory(newHistory);
    } catch (error) {
      console.error('Failed to save request history:', error);
    }
  };

  // Filter endpoints by search query and tag
  const filteredEndpoints = useMemo(() => {
    return API_ENDPOINTS.filter((endpoint) => {
      const matchesSearch = searchQuery === '' ||
        endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.summary?.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.tag.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesTag = !selectedTag || endpoint.tag === selectedTag;
      return matchesSearch && matchesTag;
    });
  }, [searchQuery, selectedTag]);

  // Group endpoints by tag
  const endpointsByTag = useMemo(() => {
    const grouped: Record<string, ApiEndpoint[]> = {};
    filteredEndpoints.forEach((endpoint) => {
      if (!grouped[endpoint.tag]) {
        grouped[endpoint.tag] = [];
      }
      grouped[endpoint.tag].push(endpoint);
    });
    return grouped;
  }, [filteredEndpoints]);

  // Get unique tags
  const tags = useMemo(() => {
    return Array.from(new Set(API_ENDPOINTS.map((e) => e.tag))).sort();
  }, []);

  // Toggle tag expansion
  const toggleTag = (tag: string) => {
    const newExpanded = new Set(expandedTags);
    if (newExpanded.has(tag)) {
      newExpanded.delete(tag);
    } else {
      newExpanded.add(tag);
    }
    setExpandedTags(newExpanded);
  };

  // Execute API request
  const executeRequest = async (method: Method, url: string, headers?: any, body?: any) => {
    setIsLoading(true);
    setResponse(null);

    const startTime = Date.now();
    const entryId = `${Date.now()}-${Math.random()}`;

    try {
      const parsedHeaders = headers && typeof headers === 'string' ? JSON.parse(headers) : headers || {};
      const parsedBody = body && typeof body === 'string' ? JSON.parse(body) : body;

      const fullUrl = url.startsWith('http') ? url : `http://localhost:8871${url}`;

      const response = await axios({
        method,
        url: fullUrl,
        headers: parsedHeaders,
        data: parsedBody,
      });

      const duration = Date.now() - startTime;

      const entry: RequestHistoryEntry = {
        id: entryId,
        timestamp: startTime,
        method,
        url,
        headers: parsedHeaders,
        body: parsedBody,
        status: response.status,
        responseBody: response.data,
        duration,
      };

      saveHistory([entry, ...history]);
      setResponse({
        status: response.status,
        statusText: response.statusText,
        headers: response.headers,
        data: response.data,
        duration,
      });

      showToast(`Request successful (${response.status})`, 'success');
    } catch (error: any) {
      const duration = Date.now() - startTime;
      const entry: RequestHistoryEntry = {
        id: entryId,
        timestamp: startTime,
        method,
        url,
        headers: headers ? JSON.parse(headers) : undefined,
        body: body ? JSON.parse(body) : undefined,
        status: error.response?.status,
        error: error.message,
        responseBody: error.response?.data,
        duration,
      };

      saveHistory([entry, ...history]);
      setResponse({
        status: error.response?.status || 0,
        statusText: error.response?.statusText || 'Error',
        headers: error.response?.headers,
        data: error.response?.data || { error: error.message },
        duration,
      });

      showToast(`Request failed: ${error.message}`, 'error');
    } finally {
      setIsLoading(false);
    }
  };

  // Execute selected endpoint
  const executeSelectedEndpoint = () => {
    if (!selectedEndpoint) return;
    executeRequest(selectedEndpoint.method, selectedEndpoint.path);
  };

  // Execute custom request
  const executeCustomRequest = () => {
    if (!customUrl.trim()) {
      showToast('Please enter a URL', 'error');
      return;
    }
    executeRequest(customMethod, customUrl, customHeaders, customBody);
  };

  // Load request from history
  const loadHistoryEntry = (entry: RequestHistoryEntry) => {
    setViewMode('explorer');
    setCustomMethod(entry.method);
    setCustomUrl(entry.url);
    setCustomHeaders(JSON.stringify(entry.headers || {}, null, 2));
    setCustomBody(JSON.stringify(entry.body || {}, null, 2));
    setResponse({
      status: entry.status,
      statusText: entry.error ? 'Error' : 'OK',
      data: entry.responseBody,
      duration: entry.duration,
    });
  };

  // Clear history
  const clearHistory = () => {
    localStorage.removeItem(STORAGE_KEY);
    setHistory([]);
    showToast('History cleared', 'success');
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="border-b p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold">API Explorer</h2>
          <div className="flex gap-2">
            <button
              onClick={() => setViewMode('explorer')}
              className={`px-3 py-1.5 rounded-md text-sm transition-colors ${
                viewMode === 'explorer' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
              }`}
            >
              Explorer
            </button>
            <button
              onClick={() => setViewMode('history')}
              className={`px-3 py-1.5 rounded-md text-sm transition-colors ${
                viewMode === 'history' ? 'bg-primary text-primary-foreground' : 'bg-muted hover:bg-muted/80'
              }`}
            >
              <Clock className="h-4 w-4 inline-block mr-1" />
              History ({history.length})
            </button>
            <a
              href="http://localhost:8871/swagger-ui"
              target="_blank"
              rel="noopener noreferrer"
              className="px-3 py-1.5 rounded-md text-sm bg-muted hover:bg-muted/80 transition-colors"
            >
              <ExternalLink className="h-4 w-4 inline-block mr-1" />
              Swagger UI
            </a>
          </div>
        </div>

        {viewMode === 'explorer' && (
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <input
                type="text"
                placeholder="Search endpoints..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 rounded-md border bg-background"
              />
            </div>
            {selectedTag && (
              <button
                onClick={() => setSelectedTag(null)}
                className="px-3 py-2 rounded-md border bg-background text-sm"
              >
                Clear filter: {selectedTag}
              </button>
            )}
          </div>
        )}
      </div>

      <div className="flex-1 overflow-hidden flex">
        {/* Left panel - Endpoint list or history */}
        <div className="w-80 border-r overflow-y-auto">
          {viewMode === 'explorer' && (
            <div className="p-2">
              {Object.entries(endpointsByTag).map(([tag, endpoints]) => (
                <div key={tag} className="mb-2">
                  <button
                    onClick={() => toggleTag(tag)}
                    className="w-full flex items-center gap-2 px-3 py-2 rounded-md hover:bg-muted transition-colors text-left"
                  >
                    {expandedTags.has(tag) ? (
                      <ChevronDown className="h-4 w-4" />
                    ) : (
                      <ChevronRight className="h-4 w-4" />
                    )}
                    <span className="font-semibold capitalize">{tag}</span>
                    <span className="text-xs text-muted-foreground ml-auto">
                      {endpoints.length}
                    </span>
                  </button>

                  {expandedTags.has(tag) && (
                    <div className="ml-2 space-y-1">
                      {endpoints.map((endpoint, idx) => (
                        <button
                          key={idx}
                          onClick={() => {
                            setSelectedEndpoint(endpoint);
                            setCustomMethod(endpoint.method);
                            setCustomUrl(endpoint.path);
                          }}
                          className={`w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors text-left ${
                            selectedEndpoint === endpoint
                              ? 'bg-primary/10 border border-primary'
                              : 'hover:bg-muted'
                          }`}
                        >
                          <span
                            className={`px-2 py-0.5 rounded text-xs font-medium text-white ${
                              METHOD_COLORS[endpoint.method]
                            }`}
                          >
                            {endpoint.method}
                          </span>
                          <span className="flex-1 truncate text-xs font-mono">
                            {endpoint.path}
                          </span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {viewMode === 'history' && (
            <div className="p-2">
              {history.length > 0 && (
                <button
                  onClick={clearHistory}
                  className="w-full mb-2 px-3 py-2 rounded-md bg-destructive/10 text-destructive hover:bg-destructive/20 transition-colors text-sm flex items-center justify-center gap-2"
                >
                  <Trash2 className="h-4 w-4" />
                  Clear History
                </button>
              )}

              {history.length === 0 ? (
                <p className="text-center text-muted-foreground text-sm py-8">
                  No requests yet
                </p>
              ) : (
                <div className="space-y-2">
                  {history.map((entry) => (
                    <button
                      key={entry.id}
                      onClick={() => loadHistoryEntry(entry)}
                      className="w-full p-3 rounded-md border hover:bg-muted transition-colors text-left"
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span
                          className={`px-2 py-0.5 rounded text-xs font-medium text-white ${
                            METHOD_COLORS[entry.method]
                          }`}
                        >
                          {entry.method}
                        </span>
                        <span
                          className={`px-2 py-0.5 rounded text-xs font-medium ${
                            entry.status && entry.status >= 200 && entry.status < 300
                              ? 'bg-green-100 text-green-800'
                              : 'bg-red-100 text-red-800'
                          }`}
                        >
                          {entry.status || 'ERR'}
                        </span>
                        {entry.duration && (
                          <span className="text-xs text-muted-foreground">
                            {entry.duration}ms
                          </span>
                        )}
                      </div>
                      <p className="text-xs font-mono truncate">{entry.url}</p>
                      <p className="text-xs text-muted-foreground">
                        {new Date(entry.timestamp).toLocaleString()}
                      </p>
                    </button>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {/* Right panel - Request builder and response */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Request builder */}
          <div className="border-b p-4 space-y-4">
            <div>
              <label className="block text-sm font-medium mb-2">Request</label>
              <div className="flex gap-2">
                <select
                  value={customMethod}
                  onChange={(e) => setCustomMethod(e.target.value as Method)}
                  className="px-3 py-2 rounded-md border bg-background"
                >
                  <option value="GET">GET</option>
                  <option value="POST">POST</option>
                  <option value="PUT">PUT</option>
                  <option value="PATCH">PATCH</option>
                  <option value="DELETE">DELETE</option>
                </select>
                <input
                  type="text"
                  placeholder="/api/v1/..."
                  value={customUrl}
                  onChange={(e) => setCustomUrl(e.target.value)}
                  className="flex-1 px-3 py-2 rounded-md border bg-background font-mono text-sm"
                />
                <button
                  onClick={executeCustomRequest}
                  disabled={isLoading}
                  className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 disabled:opacity-50 transition-colors flex items-center gap-2"
                >
                  <Send className="h-4 w-4" />
                  Send
                </button>
              </div>
            </div>

            {customMethod !== 'GET' && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-2">Headers (JSON)</label>
                  <textarea
                    value={customHeaders}
                    onChange={(e) => setCustomHeaders(e.target.value)}
                    className="w-full px-3 py-2 rounded-md border bg-background font-mono text-sm"
                    rows={3}
                    placeholder='{"Content-Type": "application/json"}'
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium mb-2">Body (JSON)</label>
                  <textarea
                    value={customBody}
                    onChange={(e) => setCustomBody(e.target.value)}
                    className="w-full px-3 py-2 rounded-md border bg-background font-mono text-sm"
                    rows={5}
                    placeholder='{"key": "value"}'
                  />
                </div>
              </>
            )}
          </div>

          {/* Response */}
          <div className="flex-1 overflow-y-auto p-4">
            {response ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">Response</h3>
                  <div className="flex items-center gap-2">
                    <span
                      className={`px-2 py-1 rounded text-sm font-medium ${
                        response.status >= 200 && response.status < 300
                          ? 'bg-green-100 text-green-800'
                          : 'bg-red-100 text-red-800'
                      }`}
                    >
                      {response.status} {response.statusText}
                    </span>
                    {response.duration && (
                      <span className="text-sm text-muted-foreground">
                        {response.duration}ms
                      </span>
                    )}
                    <button
                      onClick={() => {
                        copyToClipboard(JSON.stringify(response.data, null, 2));
                        showToast('Response copied to clipboard', 'success');
                      }}
                      className="p-2 rounded-md hover:bg-muted transition-colors"
                      title="Copy response"
                    >
                      <Copy className="h-4 w-4" />
                    </button>
                  </div>
                </div>

                <div className="bg-muted rounded-md p-4 overflow-x-auto">
                  <pre className="text-sm font-mono">
                    {JSON.stringify(response.data, null, 2)}
                  </pre>
                </div>
              </div>
            ) : isLoading ? (
              <div className="flex items-center justify-center h-64">
                <div className="text-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4" />
                  <p className="text-muted-foreground">Sending request...</p>
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-center h-64">
                <p className="text-muted-foreground">
                  Select an endpoint or enter a custom request to get started
                </p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
