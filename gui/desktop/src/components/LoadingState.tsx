/**
 * Loading State Component - M7 Polish & UX
 *
 * Reusable loading indicators with ARIA support
 */

import { Loader2 } from 'lucide-react';

interface LoadingStateProps {
  message?: string;
  size?: 'sm' | 'md' | 'lg';
  fullScreen?: boolean;
}

export function LoadingState({ message = 'Loading...', size = 'md', fullScreen = false }: LoadingStateProps) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-8 w-8',
    lg: 'h-12 w-12',
  };

  const content = (
    <div className="flex flex-col items-center justify-center gap-3">
      <Loader2 className={`${sizeClasses[size]} animate-spin text-primary`} />
      <p className="text-sm text-muted-foreground">{message}</p>
    </div>
  );

  if (fullScreen) {
    return (
      <div
        className="fixed inset-0 bg-background/80 backdrop-blur-sm flex items-center justify-center z-50"
        role="status"
        aria-live="polite"
        aria-label={message}
      >
        {content}
      </div>
    );
  }

  return (
    <div className="flex items-center justify-center py-12" role="status" aria-live="polite" aria-label={message}>
      {content}
    </div>
  );
}

/**
 * Loading Skeleton - For content placeholders
 */
export function LoadingSkeleton({ className = '' }: { className?: string }) {
  return <div className={`animate-pulse bg-muted rounded ${className}`} aria-hidden="true" />;
}

/**
 * Loading Spinner - Inline spinner
 */
export function LoadingSpinner({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8',
  };

  return <Loader2 className={`${sizeClasses[size]} animate-spin`} aria-label="Loading" />;
}
