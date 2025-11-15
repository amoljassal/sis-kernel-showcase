/**
 * Adapter version control panel with Git-like version tree and management
 */

import { useState, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  versionsApi,
  AdapterVersion,
  CommitVersionRequest,
} from '../lib/api';
import {
  GitBranch,
  GitCommit,
  Tag,
  RotateCcw,
  Trash2,
  Clock,
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  Hash,
  FolderOpen,
} from 'lucide-react';

export function VersionsPanel() {
  const queryClient = useQueryClient();
  const tableRef = useRef<HTMLDivElement>(null);

  const [commitDesc, setCommitDesc] = useState('');
  const [commitEnv, setCommitEnv] = useState('development');
  const [tagVersionId, setTagVersionId] = useState('');
  const [tagName, setTagName] = useState('');
  const [rollbackVersionId, setRollbackVersionId] = useState('');
  const [gcKeepCount, setGcKeepCount] = useState('10');

  // Fetch version list
  const { data: versionsData, error: versionsError } = useQuery({
    queryKey: ['versions', 'list'],
    queryFn: () => versionsApi.getList({ limit: 50 }),
    refetchInterval: 5000,
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  const currentVersion = versionsData?.current_version;
  const versions = versionsData?.versions ?? [];

  // Virtualizer for versions table
  const rowVirtualizer = useVirtualizer({
    count: versions.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 96,
    overscan: 10,
  });

  // Mutations
  const commitMutation = useMutation({
    mutationFn: (req: CommitVersionRequest) => versionsApi.commit(req),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions'] });
      setCommitDesc('');
    },
  });

  const rollbackMutation = useMutation({
    mutationFn: (versionId: number) => versionsApi.rollback(versionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions'] });
      setRollbackVersionId('');
    },
  });

  const tagMutation = useMutation({
    mutationFn: (req: { version_id: number; tag: string }) => versionsApi.tag(req.version_id, req.tag),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions'] });
      setTagVersionId('');
      setTagName('');
    },
  });

  const gcMutation = useMutation({
    mutationFn: (keepCount: number) => versionsApi.garbageCollect(keepCount),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['versions'] });
    },
  });

  // Handle commit
  const handleCommit = () => {
    if (commitDesc.trim()) {
      commitMutation.mutate({
        description: commitDesc,
        environment_tag: commitEnv,
        metadata: {},
      });
    }
  };

  // Handle rollback
  const handleRollback = () => {
    const versionId = parseInt(rollbackVersionId, 10);
    if (!isNaN(versionId)) {
      rollbackMutation.mutate(versionId);
    }
  };

  // Handle tag
  const handleTag = () => {
    const versionId = parseInt(tagVersionId, 10);
    if (!isNaN(versionId) && tagName.trim()) {
      tagMutation.mutate({ version_id: versionId, tag: tagName });
    }
  };

  // Handle garbage collection
  const handleGC = () => {
    const keep = parseInt(gcKeepCount, 10);
    if (!isNaN(keep) && keep > 0) {
      gcMutation.mutate(keep);
    }
  };

  // Helper to format file size
  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${(ms / 60000).toFixed(1)}m`;
  };

  if (versionsError) {
    return (
      <div className="p-6">
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2 text-red-800">
            <AlertTriangle className="h-5 w-5" />
            <span className="font-medium">Failed to load version control data</span>
          </div>
          <p className="mt-2 text-sm text-red-700">
            {versionsError instanceof Error ? versionsError.message : 'Unknown error'}
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
          <GitBranch className="h-6 w-6 text-purple-600" />
          <h2 className="text-2xl font-bold text-gray-900">Adapter Version Control</h2>
        </div>
        <p className="mt-1 text-sm text-gray-600">
          Git-like versioning for ML model adapters
        </p>
      </div>

      {/* Current Version Info */}
      {currentVersion !== undefined && versions.length > 0 && (
        <div className="p-6 bg-white border-b border-gray-200">
          <div className="flex items-center gap-2 mb-4">
            <GitCommit className="h-5 w-5 text-green-600" />
            <h3 className="text-lg font-semibold text-gray-900">Current Version: v{currentVersion}</h3>
          </div>

          {(() => {
            const current = versions.find(v => v.version_id === currentVersion);
            if (!current) return null;

            return (
              <div className="bg-gray-50 rounded-lg p-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-600">Description:</span>
                      <span className="font-medium text-gray-900">{current.description}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Timestamp:</span>
                      <span className="font-medium text-gray-900">
                        {new Date(current.timestamp).toLocaleString()}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Environment:</span>
                      <span className="font-medium text-gray-900">{current.metadata.environment_tag}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Hash:</span>
                      <span className="font-mono text-xs text-gray-600">{current.hash.slice(0, 12)}</span>
                    </div>
                  </div>

                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-600">Training Examples:</span>
                      <span className="font-medium text-gray-900">
                        {current.metadata.training_examples.toLocaleString()}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Training Duration:</span>
                      <span className="font-medium text-gray-900">
                        {formatDuration(current.metadata.training_duration_ms)}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Final Loss:</span>
                      <span className="font-medium text-gray-900">{current.metadata.final_loss.toFixed(4)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600">Accuracy Improvement:</span>
                      <span className={`font-medium ${
                        current.metadata.accuracy_improvement >= 0 ? 'text-green-600' : 'text-red-600'
                      }`}>
                        {current.metadata.accuracy_improvement > 0 ? '+' : ''}
                        {(current.metadata.accuracy_improvement * 100).toFixed(2)}%
                      </span>
                    </div>
                  </div>
                </div>

                {current.tags.length > 0 && (
                  <div className="mt-3 pt-3 border-t border-gray-200">
                    <div className="flex items-center gap-2 flex-wrap">
                      <Tag className="h-4 w-4 text-purple-600" />
                      {current.tags.map((tag) => (
                        <span
                          key={tag}
                          className="px-2 py-1 text-xs font-medium bg-purple-100 text-purple-800 rounded"
                        >
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            );
          })()}
        </div>
      )}

      {/* Version Controls */}
      <div className="p-6 bg-white border-b border-gray-200">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* Commit New Version */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
              <GitCommit className="h-4 w-4" />
              Commit New Version
            </h4>
            <div className="space-y-2">
              <input
                type="text"
                placeholder="Version description..."
                value={commitDesc}
                onChange={(e) => setCommitDesc(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
              />
              <select
                value={commitEnv}
                onChange={(e) => setCommitEnv(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-purple-500"
              >
                <option value="development">Development</option>
                <option value="staging">Staging</option>
                <option value="production">Production</option>
              </select>
              <button
                onClick={handleCommit}
                disabled={!commitDesc.trim() || commitMutation.isPending}
                className="w-full px-3 py-2 bg-purple-600 text-white text-sm rounded-md hover:bg-purple-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                Commit
              </button>
            </div>
          </div>

          {/* Rollback to Version */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
              <RotateCcw className="h-4 w-4" />
              Rollback
            </h4>
            <div className="space-y-2">
              <input
                type="number"
                placeholder="Version ID..."
                value={rollbackVersionId}
                onChange={(e) => setRollbackVersionId(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-orange-500"
              />
              <button
                onClick={handleRollback}
                disabled={!rollbackVersionId || rollbackMutation.isPending}
                className="w-full px-3 py-2 bg-orange-600 text-white text-sm rounded-md hover:bg-orange-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                Rollback
              </button>
            </div>
          </div>

          {/* Tag Version */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
              <Tag className="h-4 w-4" />
              Tag Version
            </h4>
            <div className="space-y-2">
              <input
                type="number"
                placeholder="Version ID..."
                value={tagVersionId}
                onChange={(e) => setTagVersionId(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <input
                type="text"
                placeholder="Tag name..."
                value={tagName}
                onChange={(e) => setTagName(e.target.value)}
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <button
                onClick={handleTag}
                disabled={!tagVersionId || !tagName.trim() || tagMutation.isPending}
                className="w-full px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                Add Tag
              </button>
            </div>
          </div>

          {/* Garbage Collection */}
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-medium text-gray-900 mb-3 flex items-center gap-2">
              <Trash2 className="h-4 w-4" />
              Garbage Collect
            </h4>
            <div className="space-y-2">
              <input
                type="number"
                placeholder="Keep count..."
                value={gcKeepCount}
                onChange={(e) => setGcKeepCount(e.target.value)}
                min="1"
                className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-red-500"
              />
              <button
                onClick={handleGC}
                disabled={gcMutation.isPending}
                className="w-full px-3 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 disabled:bg-gray-300 disabled:cursor-not-allowed"
              >
                Run GC
              </button>
              <p className="text-xs text-gray-600">
                Remove old versions, keep last {gcKeepCount}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Version History Tree */}
      <div className="flex-1 flex flex-col p-6 overflow-hidden">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Version History</h3>

        <div
          ref={tableRef}
          className="flex-1 overflow-auto bg-white border border-gray-200 rounded-lg"
          style={{ contain: 'strict' }}
        >
          <div style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const version = versions[virtualRow.index];
              const isCurrent = version.version_id === currentVersion;

              return (
                <div
                  key={version.version_id}
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: '100%',
                    height: `${virtualRow.size}px`,
                    transform: `translateY(${virtualRow.start}px)`,
                  }}
                  className={`border-b border-gray-100 hover:bg-gray-50 px-4 py-3 ${
                    isCurrent ? 'bg-green-50' : ''
                  }`}
                >
                  <div className="flex items-start gap-4">
                    {/* Tree visualization */}
                    <div className="flex flex-col items-center">
                      {isCurrent ? (
                        <CheckCircle className="h-5 w-5 text-green-600" />
                      ) : (
                        <GitCommit className="h-5 w-5 text-gray-400" />
                      )}
                      {virtualRow.index < versions.length - 1 && (
                        <div className="w-0.5 flex-1 bg-gray-300 mt-1" style={{ minHeight: '20px' }} />
                      )}
                    </div>

                    {/* Version info */}
                    <div className="flex-1">
                      <div className="flex items-start justify-between mb-1">
                        <div className="flex items-center gap-2">
                          <span className={`text-sm font-medium ${isCurrent ? 'text-green-900' : 'text-gray-900'}`}>
                            v{version.version_id}
                          </span>
                          {isCurrent && (
                            <span className="px-2 py-0.5 text-xs font-medium bg-green-100 text-green-800 rounded">
                              CURRENT
                            </span>
                          )}
                          {version.tags.map((tag) => (
                            <span
                              key={tag}
                              className="px-2 py-0.5 text-xs font-medium bg-purple-100 text-purple-800 rounded flex items-center gap-1"
                            >
                              <Tag className="h-3 w-3" />
                              {tag}
                            </span>
                          ))}
                        </div>
                        <span className="text-xs text-gray-500">
                          {new Date(version.timestamp).toLocaleString()}
                        </span>
                      </div>

                      <div className="text-sm text-gray-900 mb-2">{version.description}</div>

                      <div className="flex items-center gap-4 text-xs text-gray-600">
                        <span className="flex items-center gap-1">
                          <Hash className="h-3 w-3" />
                          {version.hash.slice(0, 8)}
                        </span>
                        {version.parent_version !== null && (
                          <span>Parent: v{version.parent_version}</span>
                        )}
                        <span className="flex items-center gap-1">
                          <TrendingUp className="h-3 w-3" />
                          {(version.metadata.accuracy_improvement * 100).toFixed(2)}%
                        </span>
                        <span className="flex items-center gap-1">
                          <Clock className="h-3 w-3" />
                          {formatDuration(version.metadata.training_duration_ms)}
                        </span>
                        <span className="flex items-center gap-1">
                          <FolderOpen className="h-3 w-3" />
                          {version.metadata.environment_tag}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {versions.length === 0 && (
          <div className="flex-1 flex items-center justify-center bg-white border border-gray-200 rounded-lg">
            <div className="text-center py-12">
              <GitBranch className="h-12 w-12 text-gray-400 mx-auto mb-4" />
              <p className="text-gray-600">No adapter versions yet</p>
              <p className="text-sm text-gray-500 mt-1">
                Version history will appear here when you commit new adapter versions
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
