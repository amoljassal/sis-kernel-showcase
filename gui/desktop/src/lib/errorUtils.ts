/**
 * Problem+json error utilities with actionable CTAs
 */

export interface ProblemDetail {
  type?: string;
  status?: number;
  title?: string;
  detail?: string;
  'retry-after'?: number;
}

export interface ErrorCTA {
  label: string;
  action: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
}

export interface EnhancedError {
  message: string;
  detail?: string;
  retryAfter?: number;
  requestId?: string;
  ctas?: ErrorCTA[];
}

/**
 * Parse error response and generate actionable CTAs
 */
export function parseErrorWithCTAs(
  error: any,
  context?: {
    onStopReplay?: () => void;
    onStopQemu?: () => void;
    onStartQemu?: () => void;
    onSwitchToReplay?: () => void;
    onFocusField?: (fieldName: string) => void;
  }
): EnhancedError {
  // Extract problem+json details
  const problemDetail: ProblemDetail = error?.response?.data || {};
  const type = problemDetail.type || '';
  const detail = problemDetail.detail || error?.message || 'An error occurred';
  const retryAfter = problemDetail['retry-after'];
  const requestId = error?.requestId || error?.response?.data?.requestId;

  // Default enhanced error
  const enhanced: EnhancedError = {
    message: problemDetail.title || 'Error',
    detail,
    retryAfter,
    requestId,
    ctas: [],
  };

  // Map problem types to CTAs
  if (type.includes('/errors/busy')) {
    enhanced.message = 'Resource Busy';
    enhanced.ctas = [];

    // Check if detail mentions replay or QEMU
    if (detail.toLowerCase().includes('replay')) {
      if (context?.onStopReplay) {
        enhanced.ctas.push({
          label: 'Stop Replay',
          action: context.onStopReplay,
          variant: 'primary',
        });
      }
    } else if (detail.toLowerCase().includes('qemu')) {
      if (context?.onStopQemu) {
        enhanced.ctas.push({
          label: 'Stop QEMU',
          action: context.onStopQemu,
          variant: 'danger',
        });
      }
    }

    // Show retry-after hint
    if (retryAfter) {
      enhanced.detail = `${detail}. Please retry after ${retryAfter} seconds.`;
    }
  } else if (type.includes('/errors/shell-not-ready')) {
    enhanced.message = 'Shell Not Ready';
    enhanced.ctas = [];

    if (context?.onStartQemu) {
      enhanced.ctas.push({
        label: 'Start QEMU',
        action: context.onStartQemu,
        variant: 'primary',
      });
    }

    if (context?.onSwitchToReplay) {
      enhanced.ctas.push({
        label: 'Switch to Replay',
        action: context.onSwitchToReplay,
        variant: 'secondary',
      });
    }
  } else if (type.includes('/errors/invalid-params')) {
    enhanced.message = 'Invalid Parameters';

    // Try to extract field name from detail
    const fieldMatch = detail.match(/field[:\s]+['"]?(\w+)['"]?/i);
    if (fieldMatch && context?.onFocusField) {
      const fieldName = fieldMatch[1];
      enhanced.ctas = [
        {
          label: `Fix ${fieldName}`,
          action: () => context.onFocusField!(fieldName),
          variant: 'primary',
        },
      ];
    }
  } else if (type.includes('/errors/not-found')) {
    enhanced.message = 'Resource Not Found';
  } else if (type.includes('/errors/timeout')) {
    enhanced.message = 'Request Timeout';
  } else if (type.includes('/errors/conflict')) {
    enhanced.message = 'Conflict';
  }

  return enhanced;
}

/**
 * Get error variant color class
 */
export function getErrorVariantClass(variant?: 'primary' | 'secondary' | 'danger'): string {
  switch (variant) {
    case 'primary':
      return 'bg-primary text-primary-foreground hover:bg-primary/90';
    case 'secondary':
      return 'bg-muted text-foreground hover:bg-muted-foreground/10';
    case 'danger':
      return 'bg-destructive text-destructive-foreground hover:bg-destructive/90';
    default:
      return 'bg-primary text-primary-foreground hover:bg-primary/90';
  }
}
