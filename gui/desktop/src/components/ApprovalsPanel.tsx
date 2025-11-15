/**
 * Memory approvals panel with pending operations and actions
 */

import { useState, useRef } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  memoryApi,
  MemoryApprovalStatus,
  PendingOperation,
} from '../lib/api';
import {
  Check,
  X,
  AlertTriangle,
  Clock,
  Shield,
  Database,
} from 'lucide-react';

export function ApprovalsPanel() {
  const queryClient = useQueryClient();
  const [approveCount, setApproveCount] = useState<string>('1');
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const [showConfirm, setShowConfirm] = useState<{ action: string; id?: string } | null>(null);
  const tableRef = useRef<HTMLDivElement>(null);

  // Fetch approval status
  const { data: status } = useQuery({
    queryKey: ['memory', 'status'],
    queryFn: () => memoryApi.toggleApproval('status'),
    refetchInterval: 2000,
  });

  // Fetch pending approvals
  const { data: pending = [] } = useQuery({
    queryKey: ['memory', 'approvals'],
    queryFn: () => memoryApi.getApprovals(100),
    refetchInterval: 3000,
    enabled: status?.enabled === true,
  });

  // Mutations
  const toggleApproval = useMutation({
    mutationFn: (action: 'on' | 'off') => memoryApi.toggleApproval(action),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memory'] });
    },
  });

  const approve = useMutation({
    mutationFn: (n: number) => memoryApi.approve(n),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memory'] });
      setSelectedIds(new Set());
    },
  });

  const reject = useMutation({
    mutationFn: (id?: string) => memoryApi.reject(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memory'] });
      setSelectedIds(new Set());
      setShowConfirm(null);
    },
  });

  // Virtualizer
  const rowVirtualizer = useVirtualizer({
    count: pending.length,
    getScrollElement: () => tableRef.current,
    estimateSize: () => 56,
    overscan: 10,
  });

  // Handle approve N
  const handleApproveN = () => {
    const n = parseInt(approveCount, 10);
    if (!isNaN(n) && n > 0) {
      approve.mutate(n);
    }
  };

  // Handle approve selected
  const handleApproveSelected = () => {
    if (selectedIds.size > 0) {
      // Approve operations in order they appear
      approve.mutate(selectedIds.size);
    }
  };

  // Handle reject
  const handleReject = (id?: string) => {
    setShowConfirm({ action: id ? 'reject_one' : 'reject_all', id });
  };

  const confirmReject = () => {
    if (showConfirm) {
      reject.mutate(showConfirm.id);
    }
  };

  // Toggle selection
  const toggleSelect = (id: string) => {
    const newSelected = new Set(selectedIds);
    if (newSelected.has(id)) {
      newSelected.delete(id);
    } else {
      newSelected.add(id);
    }
    setSelectedIds(newSelected);
  };

  // Risk color
  const getRiskColor = (risk: string) => {
    switch (risk.toLowerCase()) {
      case 'low':
        return 'text-green-500 bg-green-500/10';
      case 'medium':
        return 'text-yellow-500 bg-yellow-500/10';
      case 'high':
        return 'text-red-500 bg-red-500/10';
      default:
        return 'text-muted-foreground bg-muted';
    }
  };

  return (
    <div className="h-full flex flex-col bg-card rounded-lg border">
      {/* Header */}
      <div className="p-4 border-b">
        <h3 className="text-lg font-semibold flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Memory Approvals
        </h3>
      </div>

      {/* Controls */}
      <div className="p-4 border-b space-y-3">
        {/* Toggle */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">Approval Mode</div>
            <div className="text-xs text-muted-foreground">
              Require manual approval for memory operations
            </div>
          </div>
          <button
            onClick={() => toggleApproval.mutate(status?.enabled ? 'off' : 'on')}
            disabled={toggleApproval.isPending}
            className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              status?.enabled
                ? 'bg-green-500 text-white hover:bg-green-600'
                : 'bg-muted text-muted-foreground hover:bg-muted/80'
            }`}
          >
            {status?.enabled ? 'Enabled' : 'Disabled'}
          </button>
        </div>

        {/* Status Cards */}
        {status && (
          <div className="grid grid-cols-3 gap-2">
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Pending</div>
              <div className="text-lg font-semibold">{status.pending_count}</div>
            </div>
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Approved</div>
              <div className="text-lg font-semibold text-green-500">{status.total_approved}</div>
            </div>
            <div className="bg-background rounded p-2">
              <div className="text-xs text-muted-foreground mb-1">Rejected</div>
              <div className="text-lg font-semibold text-red-500">{status.total_rejected}</div>
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex gap-2">
          <div className="flex gap-1 items-center">
            <input
              type="number"
              value={approveCount}
              onChange={(e) => setApproveCount(e.target.value)}
              className="w-16 px-2 py-1 bg-background border rounded text-sm"
              min="1"
            />
            <button
              onClick={handleApproveN}
              disabled={approve.isPending || pending.length === 0}
              className="px-3 py-1 bg-green-500 text-white rounded text-sm hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
            >
              <Check className="h-4 w-4" />
              Approve N
            </button>
          </div>

          <button
            onClick={handleApproveSelected}
            disabled={approve.isPending || selectedIds.size === 0}
            className="px-3 py-1 bg-green-500 text-white rounded text-sm hover:bg-green-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          >
            <Check className="h-4 w-4" />
            Approve Selected ({selectedIds.size})
          </button>

          <button
            onClick={() => handleReject()}
            disabled={reject.isPending || pending.length === 0}
            className="px-3 py-1 bg-red-500 text-white rounded text-sm hover:bg-red-600 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
          >
            <X className="h-4 w-4" />
            Reject All
          </button>
        </div>
      </div>

      {/* Pending Operations Table */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="px-4 py-2 bg-muted/50 border-b">
          <div className="flex items-center justify-between text-xs font-semibold">
            <div className="w-8">
              <input
                type="checkbox"
                checked={selectedIds.size === pending.length && pending.length > 0}
                onChange={(e) => {
                  if (e.target.checked) {
                    setSelectedIds(new Set(pending.map(op => op.id)));
                  } else {
                    setSelectedIds(new Set());
                  }
                }}
                className="cursor-pointer"
              />
            </div>
            <div className="w-24">ID</div>
            <div className="w-24">Type</div>
            <div className="w-20 text-right">Confidence</div>
            <div className="w-20">Risk</div>
            <div className="flex-1">Reason</div>
            <div className="w-32">Timestamp</div>
            <div className="w-20">Actions</div>
          </div>
        </div>

        <div
          ref={tableRef}
          className="flex-1 overflow-y-auto"
          style={{ contain: 'strict' }}
        >
          {pending.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
              {status?.enabled ? 'No pending operations' : 'Enable approval mode to see pending operations'}
            </div>
          ) : (
            <div
              style={{
                height: `${rowVirtualizer.getTotalSize()}px`,
                width: '100%',
                position: 'relative',
              }}
            >
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const op = pending[virtualRow.index];
                const isSelected = selectedIds.has(op.id);

                return (
                  <div
                    key={virtualRow.key}
                    className={`absolute top-0 left-0 w-full px-4 py-2 transition-colors border-b ${
                      isSelected ? 'bg-primary/10' : 'hover:bg-muted/50'
                    }`}
                    style={{
                      height: `${virtualRow.size}px`,
                      transform: `translateY(${virtualRow.start}px)`,
                    }}
                  >
                    <div className="flex items-center justify-between text-xs">
                      {/* Checkbox */}
                      <div className="w-8">
                        <input
                          type="checkbox"
                          checked={isSelected}
                          onChange={() => toggleSelect(op.id)}
                          className="cursor-pointer"
                        />
                      </div>

                      {/* ID */}
                      <div className="w-24 truncate font-mono" title={op.id}>
                        {op.id.substring(0, 8)}...
                      </div>

                      {/* Type */}
                      <div className="w-24 flex items-center gap-1">
                        <Database className="h-3 w-3" />
                        {op.op_type}
                      </div>

                      {/* Confidence */}
                      <div className="w-20 text-right font-semibold">
                        {(op.confidence * 100).toFixed(0)}%
                      </div>

                      {/* Risk */}
                      <div className="w-20">
                        <span className={`px-2 py-0.5 rounded text-xs font-medium ${getRiskColor(op.risk)}`}>
                          {op.risk}
                        </span>
                      </div>

                      {/* Reason */}
                      <div className="flex-1 truncate" title={op.reason}>
                        {op.reason}
                      </div>

                      {/* Timestamp */}
                      <div className="w-32 text-muted-foreground flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {new Date(op.timestamp / 1000).toLocaleTimeString()}
                      </div>

                      {/* Actions */}
                      <div className="w-20 flex gap-1">
                        <button
                          onClick={() => approve.mutate(1)}
                          className="p-1 hover:bg-green-500/20 rounded"
                          title="Approve"
                        >
                          <Check className="h-4 w-4 text-green-500" />
                        </button>
                        <button
                          onClick={() => handleReject(op.id)}
                          className="p-1 hover:bg-red-500/20 rounded"
                          title="Reject"
                        >
                          <X className="h-4 w-4 text-red-500" />
                        </button>
                      </div>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>

      {/* Confirm Dialog */}
      {showConfirm && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card border rounded-lg p-6 max-w-md">
            <div className="flex items-start gap-3 mb-4">
              <AlertTriangle className="h-6 w-6 text-yellow-500 mt-0.5" />
              <div>
                <h4 className="font-semibold mb-1">Confirm Reject</h4>
                <p className="text-sm text-muted-foreground">
                  {showConfirm.action === 'reject_all'
                    ? `Are you sure you want to reject all ${pending.length} pending operations?`
                    : 'Are you sure you want to reject this operation?'}
                </p>
              </div>
            </div>
            <div className="flex gap-2 justify-end">
              <button
                onClick={() => setShowConfirm(null)}
                className="px-4 py-2 bg-muted text-foreground rounded hover:bg-muted/80"
              >
                Cancel
              </button>
              <button
                onClick={confirmReject}
                className="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
              >
                Reject
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
