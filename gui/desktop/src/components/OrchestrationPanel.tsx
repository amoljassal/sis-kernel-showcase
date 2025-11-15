/**
 * Multi-agent orchestration panel with stats, agent cards, and decision history
 */

import { useState, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  orchestratorApi,
  OrchestrationStats,
  AgentInfo,
  CoordinatedDecision,
} from '../lib/api';
import {
  Users,
  CheckCircle,
  TrendingUp,
  AlertTriangle,
  XCircle,
  Activity,
  Clock,
  Shield,
  Brain,
  Zap,
} from 'lucide-react';

export function OrchestrationPanel() {
  const tableRef = useRef<HTMLDivElement>(null);
  const [decisionTypeFilter, setDecisionTypeFilter] = useState<string>('');

  // Fetch orchestration statistics
  const { data: stats, error: statsError } = useQuery({
    queryKey: ['orchestrator', 'stats'],
    queryFn: () => orchestratorApi.getStats(),
    refetchInterval: 1000, // 1 second refresh for stats
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  // Fetch agent status
  const { data: agentsData } = useQuery({
    queryKey: ['orchestrator', 'agents'],
    queryFn: () => orchestratorApi.getAgents(),
    refetchInterval: 2000, // 2 second refresh for agents
  });

  const agents = agentsData?.agents ?? [];

  // Fetch decision history
  const { data: decisionsData } = useQuery({
    queryKey: ['orchestrator', 'decisions', decisionTypeFilter],
    queryFn: () => orchestratorApi.getDecisions({
      limit: 500,
      ...(decisionTypeFilter && { type: decisionTypeFilter }),
    }),
    refetchInterval: 2000,
  });

  const decisions = decisionsData?.decisions ?? [];

  // Virtualizer for decisions table
  const rowVirtualizer = useVirtualizer({
    count: decisions.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 64,
    overscan: 10,
  });

  // Helper to get decision type badge
  const getDecisionTypeBadge = (type: CoordinatedDecision['type']) => {
    switch (type) {
      case 'unanimous':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">
            <CheckCircle className="h-3 w-3" />
            Unanimous
          </span>
        );
      case 'majority':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
            <TrendingUp className="h-3 w-3" />
            Majority
          </span>
        );
      case 'safety_override':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-orange-100 text-orange-800 rounded-full">
            <Shield className="h-3 w-3" />
            Safety Override
          </span>
        );
      case 'no_consensus':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <XCircle className="h-3 w-3" />
            No Consensus
          </span>
        );
    }
  };

  // Helper to get agent status badge
  const getAgentStatusBadge = (status: AgentInfo['status']) => {
    switch (status) {
      case 'active':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">
            <Activity className="h-3 w-3" />
            Active
          </span>
        );
      case 'inactive':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-gray-100 text-gray-800 rounded-full">
            <Clock className="h-3 w-3" />
            Inactive
          </span>
        );
      case 'error':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <AlertTriangle className="h-3 w-3" />
            Error
          </span>
        );
    }
  };

  // Helper to format latency
  const formatLatency = (us: number) => {
    if (us < 1000) return `${us}µs`;
    if (us < 1000000) return `${(us / 1000).toFixed(1)}ms`;
    return `${(us / 1000000).toFixed(2)}s`;
  };

  if (statsError) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-red-800">
            <AlertTriangle className="h-5 w-5" />
            <span className="font-medium">Failed to load orchestration data</span>
          </div>
          <p className="mt-2 text-sm text-red-700">
            {statsError instanceof Error ? statsError.message : 'Unknown error'}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 p-6">
        <div className="flex items-center gap-3">
          <Users className="h-6 w-6 text-purple-600" />
          <h2 className="text-2xl font-bold text-gray-900">Multi-Agent Orchestration</h2>
        </div>
        <p className="mt-1 text-sm text-gray-600">
          Real-time coordination and decision making across AI agents
        </p>
      </div>

      {/* Statistics Cards */}
      {stats && (
        <div className="p-6 bg-white border-b border-gray-200">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            <div className="bg-gray-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-gray-600 text-sm mb-1">
                <Activity className="h-4 w-4" />
                Total Decisions
              </div>
              <div className="text-2xl font-bold text-gray-900">{stats.total_decisions}</div>
            </div>

            <div className="bg-green-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-green-700 text-sm mb-1">
                <CheckCircle className="h-4 w-4" />
                Unanimous
              </div>
              <div className="text-2xl font-bold text-green-900">{stats.unanimous}</div>
              <div className="text-xs text-green-600 mt-1">
                {stats.total_decisions > 0 ? Math.round((stats.unanimous / stats.total_decisions) * 100) : 0}%
              </div>
            </div>

            <div className="bg-blue-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-blue-700 text-sm mb-1">
                <TrendingUp className="h-4 w-4" />
                Majority
              </div>
              <div className="text-2xl font-bold text-blue-900">{stats.majority}</div>
              <div className="text-xs text-blue-600 mt-1">
                {stats.total_decisions > 0 ? Math.round((stats.majority / stats.total_decisions) * 100) : 0}%
              </div>
            </div>

            <div className="bg-orange-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-orange-700 text-sm mb-1">
                <Shield className="h-4 w-4" />
                Safety Overrides
              </div>
              <div className="text-2xl font-bold text-orange-900">{stats.safety_overrides}</div>
              <div className="text-xs text-orange-600 mt-1">
                {stats.total_decisions > 0 ? Math.round((stats.safety_overrides / stats.total_decisions) * 100) : 0}%
              </div>
            </div>

            <div className="bg-red-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-red-700 text-sm mb-1">
                <XCircle className="h-4 w-4" />
                No Consensus
              </div>
              <div className="text-2xl font-bold text-red-900">{stats.no_consensus}</div>
              <div className="text-xs text-red-600 mt-1">
                {stats.total_decisions > 0 ? Math.round((stats.no_consensus / stats.total_decisions) * 100) : 0}%
              </div>
            </div>

            <div className="bg-purple-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-purple-700 text-sm mb-1">
                <Zap className="h-4 w-4" />
                Avg Latency
              </div>
              <div className="text-2xl font-bold text-purple-900">
                {formatLatency(stats.avg_latency_us)}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Agent Cards */}
      {agents.length > 0 && (
        <div className="p-6 bg-white border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Active Agents</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
            {agents.map((agent) => (
              <div
                key={agent.name}
                className="bg-gray-50 border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow"
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <Brain className="h-5 w-5 text-purple-600" />
                    <span className="font-medium text-gray-900">{agent.name}</span>
                  </div>
                  {getAgentStatusBadge(agent.status)}
                </div>

                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600">Type:</span>
                    <span className="font-medium text-gray-900">{agent.type}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">Priority:</span>
                    <span className="font-medium text-gray-900">{agent.priority}</span>
                  </div>

                  {agent.stats && (
                    <>
                      <div className="flex justify-between">
                        <span className="text-gray-600">Decisions:</span>
                        <span className="font-medium text-gray-900">{agent.stats.total_decisions}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">Avg Confidence:</span>
                        <span className="font-medium text-gray-900">
                          {(agent.stats.avg_confidence * 100).toFixed(1)}%
                        </span>
                      </div>
                    </>
                  )}

                  {agent.last_decision && (
                    <div className="mt-3 pt-3 border-t border-gray-200">
                      <div className="text-xs text-gray-600 mb-1">Last Decision:</div>
                      <div className="text-xs font-medium text-gray-900 truncate">
                        {agent.last_decision.action}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        {new Date(agent.last_decision.timestamp).toLocaleTimeString()}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Decision History Table */}
      <div className="flex-1 flex flex-col p-6 overflow-hidden">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Decision History</h3>
          <div className="flex items-center gap-2">
            <label htmlFor="decision-type-filter" className="text-sm text-gray-600">
              Filter:
            </label>
            <select
              id="decision-type-filter"
              value={decisionTypeFilter}
              onChange={(e) => setDecisionTypeFilter(e.target.value)}
              className="px-3 py-1 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
            >
              <option value="">All Types</option>
              <option value="unanimous">Unanimous</option>
              <option value="majority">Majority</option>
              <option value="safety_override">Safety Override</option>
              <option value="no_consensus">No Consensus</option>
            </select>
          </div>
        </div>

        <div
          ref={tableRef}
          className="flex-1 overflow-auto bg-white border border-gray-200 rounded-lg"
          style={{ contain: 'strict' }}
        >
          <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const decision = decisions[virtualRow.index];
              return (
                <div
                  key={virtualRow.index}
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: `${virtualRow.size}px`,
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                  className="border-b border-gray-100 hover:bg-gray-50 px-4 py-3"
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4 flex-1">
                      <div className="text-xs text-gray-500 w-24">
                        {new Date(decision.timestamp).toLocaleTimeString()}
                      </div>
                      {getDecisionTypeBadge(decision.type)}
                      <div className="flex-1 text-sm text-gray-900 font-medium truncate">
                        {decision.action}
                      </div>
                    </div>
                    <div className="flex items-center gap-4 text-sm">
                      {decision.confidence !== undefined && (
                        <div className="text-gray-600">
                          Confidence: <span className="font-medium">{(decision.confidence * 100).toFixed(1)}%</span>
                        </div>
                      )}
                      {decision.agents && decision.agents.length > 0 && (
                        <div className="text-gray-600">
                          Agents: <span className="font-medium">{decision.agents.join(', ')}</span>
                        </div>
                      )}
                      <div className="text-gray-500 text-xs">
                        {formatLatency(decision.latency_us)}
                      </div>
                    </div>
                  </div>
                  {decision.overridden_by && (
                    <div className="mt-2 text-xs text-orange-600 flex items-center gap-1">
                      <Shield className="h-3 w-3" />
                      Overridden by {decision.overridden_by}: {decision.reason}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>

        {decisions.length === 0 && (
          <div className="flex-1 flex items-center justify-center bg-white border border-gray-200 rounded-lg">
            <div className="text-center py-12">
              <Users className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">No coordinated decisions yet</p>
              <p className="text-sm text-gray-500 mt-1">
                Decision history will appear here once agents start making coordinated decisions
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
