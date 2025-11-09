/**
 * Conflict resolution panel with stats, priority table, and conflict history
 */

import { useState, useRef } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  conflictsApi,
  ConflictStats,
  Conflict,
  PriorityEntry,
} from '../lib/api';
import {
  AlertTriangle,
  Users,
  TrendingUp,
  Clock,
  CheckCircle,
  XCircle,
  Shield,
  Award,
} from 'lucide-react';

export function ConflictPanel() {
  const tableRef = useRef<HTMLDivElement>(null);
  const [resolvedFilter, setResolvedFilter] = useState<'all' | 'resolved' | 'unresolved'>('all');

  // Fetch conflict statistics
  const { data: stats, error: statsError } = useQuery({
    queryKey: ['conflicts', 'stats'],
    queryFn: () => conflictsApi.getStats(),
    refetchInterval: 2000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  // Fetch priority table
  const { data: priorityData } = useQuery({
    queryKey: ['conflicts', 'priorities'],
    queryFn: () => conflictsApi.getPriorityTable(),
    staleTime: Infinity, // Priority table is relatively static
  });

  const priorities = priorityData?.priorities ?? [];

  // Fetch conflict history
  const { data: conflictsData } = useQuery({
    queryKey: ['conflicts', 'history', resolvedFilter],
    queryFn: () => conflictsApi.getHistory({
      limit: 500,
      ...(resolvedFilter !== 'all' && { resolved: resolvedFilter === 'resolved' }),
    }),
    refetchInterval: 3000,
  });

  const conflicts = conflictsData?.conflicts ?? [];

  // Virtualizer for conflicts table
  const rowVirtualizer = useVirtualizer({
    count: conflicts.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 96,
    overscan: 10,
  });

  // Helper to get resolution strategy badge
  const getResolutionBadge = (strategy: Conflict['resolution']['strategy']) => {
    switch (strategy) {
      case 'priority':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full">
            <Award className="h-3 w-3" />
            Priority
          </span>
        );
      case 'voting':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-green-100 text-green-800 rounded-full">
            <Users className="h-3 w-3" />
            Voting
          </span>
        );
      case 'unresolved':
        return (
          <span className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded-full">
            <XCircle className="h-3 w-3" />
            Unresolved
          </span>
        );
    }
  };

  // Helper to format resolution time
  const formatResolutionTime = (us: number) => {
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
            <span className="font-medium">Failed to load conflict resolution data</span>
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
          <AlertTriangle className="h-6 w-6 text-orange-600" />
          <h2 className="text-2xl font-bold text-gray-900">Conflict Resolution</h2>
        </div>
        <p className="mt-1 text-sm text-gray-600">
          Agent conflict detection and resolution strategies
        </p>
      </div>

      {/* Statistics Cards */}
      {stats && (
        <div className="p-6 bg-white border-b border-gray-200">
          <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
            <div className="bg-gray-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-gray-600 text-sm mb-1">
                <AlertTriangle className="h-4 w-4" />
                Total Conflicts
              </div>
              <div className="text-2xl font-bold text-gray-900">{stats.total_conflicts}</div>
            </div>

            <div className="bg-blue-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-blue-700 text-sm mb-1">
                <Award className="h-4 w-4" />
                By Priority
              </div>
              <div className="text-2xl font-bold text-blue-900">{stats.resolved_by_priority}</div>
              <div className="text-xs text-blue-600 mt-1">
                {stats.total_conflicts > 0 ? Math.round((stats.resolved_by_priority / stats.total_conflicts) * 100) : 0}%
              </div>
            </div>

            <div className="bg-green-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-green-700 text-sm mb-1">
                <Users className="h-4 w-4" />
                By Voting
              </div>
              <div className="text-2xl font-bold text-green-900">{stats.resolved_by_voting}</div>
              <div className="text-xs text-green-600 mt-1">
                {stats.total_conflicts > 0 ? Math.round((stats.resolved_by_voting / stats.total_conflicts) * 100) : 0}%
              </div>
            </div>

            <div className="bg-red-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-red-700 text-sm mb-1">
                <XCircle className="h-4 w-4" />
                Unresolved
              </div>
              <div className="text-2xl font-bold text-red-900">{stats.unresolved}</div>
              <div className="text-xs text-red-600 mt-1">
                {stats.total_conflicts > 0 ? Math.round((stats.unresolved / stats.total_conflicts) * 100) : 0}%
              </div>
            </div>

            <div className="bg-purple-50 rounded-lg p-4">
              <div className="flex items-center gap-2 text-purple-700 text-sm mb-1">
                <Clock className="h-4 w-4" />
                Avg Resolution
              </div>
              <div className="text-2xl font-bold text-purple-900">
                {formatResolutionTime(stats.avg_resolution_time_us)}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Priority Table */}
      {priorities.length > 0 && (
        <div className="p-6 bg-white border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4 flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-600" />
            Agent Priority Table
          </h3>
          <div className="bg-gray-50 border border-gray-200 rounded-lg overflow-hidden">
            <table className="w-full">
              <thead className="bg-gray-100 border-b border-gray-200">
                <tr>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-600 uppercase tracking-wider">
                    Rank
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-600 uppercase tracking-wider">
                    Agent Name
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-600 uppercase tracking-wider">
                    Priority Score
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200">
                {priorities.map((entry, index) => (
                  <tr key={entry.agent} className="hover:bg-gray-100">
                    <td className="px-4 py-3 text-sm text-gray-600">
                      <div className="flex items-center gap-2">
                        {index === 0 && <Award className="h-4 w-4 text-yellow-500" />}
                        {index + 1}
                      </div>
                    </td>
                    <td className="px-4 py-3 text-sm font-medium text-gray-900">{entry.agent}</td>
                    <td className="px-4 py-3 text-sm text-gray-900">
                      <div className="flex items-center gap-2">
                        <div className="flex-1 bg-gray-200 rounded-full h-2 max-w-xs">
                          <div
                            className="bg-blue-600 h-2 rounded-full"
                            style={{
                              width: `${(entry.priority / Math.max(...priorities.map(p => p.priority))) * 100}%`,
                            }}
                          />
                        </div>
                        <span className="font-medium">{entry.priority}</span>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Conflict History */}
      <div className="flex-1 flex flex-col p-6 overflow-hidden">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-gray-900">Conflict History</h3>
          <div className="flex items-center gap-2">
            <label htmlFor="resolved-filter" className="text-sm text-gray-600">
              Filter:
            </label>
            <select
              id="resolved-filter"
              value={resolvedFilter}
              onChange={(e) => setResolvedFilter(e.target.value as any)}
              className="px-3 py-1 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
            >
              <option value="all">All Conflicts</option>
              <option value="resolved">Resolved Only</option>
              <option value="unresolved">Unresolved Only</option>
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
              const conflict = conflicts[virtualRow.index];
              return (
                <div
                  key={conflict.id}
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
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-3">
                      <div className="text-xs text-gray-500 w-24">
                        {new Date(conflict.timestamp).toLocaleTimeString()}
                      </div>
                      {getResolutionBadge(conflict.resolution.strategy)}
                      <div className="text-xs text-gray-500">
                        ID: {conflict.id}
                      </div>
                    </div>
                    <div className="text-xs text-gray-500">
                      {formatResolutionTime(conflict.resolution_time_us)}
                    </div>
                  </div>

                  <div className="ml-28 space-y-2">
                    {/* Conflicting agents */}
                    <div className="flex items-center gap-2 flex-wrap">
                      {conflict.agents.map((agent, idx) => (
                        <div
                          key={idx}
                          className={`text-xs px-2 py-1 rounded border ${
                            agent.agent === conflict.resolution.winner
                              ? 'bg-green-50 border-green-200 text-green-800 font-medium'
                              : 'bg-gray-50 border-gray-200 text-gray-600'
                          }`}
                        >
                          <div className="flex items-center gap-1">
                            <span>{agent.agent}</span>
                            {agent.agent === conflict.resolution.winner && (
                              <CheckCircle className="h-3 w-3" />
                            )}
                          </div>
                          <div className="text-xs">
                            {agent.action} (conf: {(agent.confidence * 100).toFixed(0)}%, pri: {agent.priority})
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* Resolution details */}
                    {conflict.resolution.strategy !== 'unresolved' && (
                      <div className="text-sm text-gray-700">
                        <span className="font-medium">Resolution:</span> {conflict.resolution.winner} chose{' '}
                        <span className="font-medium">{conflict.resolution.action}</span>
                        {' — '}
                        <span className="text-gray-600">{conflict.resolution.reason}</span>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {conflicts.length === 0 && (
          <div className="flex-1 flex items-center justify-center bg-white border border-gray-200 rounded-lg">
            <div className="text-center py-12">
              <AlertTriangle className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">No conflicts detected yet</p>
              <p className="text-sm text-gray-500 mt-1">
                Conflict history will appear here when agents propose conflicting actions
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
