/**
 * Status badge component for displaying connection and QEMU state
 */

import { QemuState } from '@/lib/api';

interface StatusBadgeProps {
  state: QemuState | 'disconnected';
}

export function StatusBadge({ state }: StatusBadgeProps) {
  const getStateStyles = () => {
    switch (state) {
      case QemuState.Running:
        return 'bg-green-500 text-white';
      case QemuState.Starting:
        return 'bg-yellow-500 text-white';
      case QemuState.Stopping:
        return 'bg-orange-500 text-white';
      case QemuState.Failed:
        return 'bg-red-500 text-white';
      case QemuState.Idle:
        return 'bg-gray-500 text-white';
      case 'disconnected':
        return 'bg-red-600 text-white';
      default:
        return 'bg-gray-400 text-white';
    }
  };

  const getStateLabel = () => {
    switch (state) {
      case 'disconnected':
        return 'Disconnected';
      default:
        return state.charAt(0).toUpperCase() + state.slice(1);
    }
  };

  return (
    <span
      className={`px-3 py-1 rounded-full text-sm font-medium ${getStateStyles()}`}
    >
      {getStateLabel()}
    </span>
  );
}
