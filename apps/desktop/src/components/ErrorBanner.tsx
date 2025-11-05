/**
 * Error Banner with Problem+json CTA support
 */

import { AlertTriangle, X } from 'lucide-react';
import { type EnhancedError, getErrorVariantClass } from '../lib/errorUtils';

interface ErrorBannerProps {
  error: EnhancedError;
  onDismiss?: () => void;
}

export function ErrorBanner({ error, onDismiss }: ErrorBannerProps) {
  return (
    <div className="bg-destructive/10 text-destructive px-4 py-3 rounded-md flex items-start gap-3">
      <AlertTriangle className="h-5 w-5 flex-shrink-0 mt-0.5" />
      <div className="flex-1 space-y-2">
        <div>
          <p className="font-semibold">{error.message}</p>
          <p className="text-sm mt-1">{error.detail}</p>
        </div>

        {/* CTAs */}
        {error.ctas && error.ctas.length > 0 && (
          <div className="flex gap-2 flex-wrap">
            {error.ctas.map((cta, index) => (
              <button
                key={index}
                onClick={cta.action}
                className={`px-3 py-1.5 rounded-md text-sm font-medium transition-colors ${getErrorVariantClass(cta.variant)}`}
              >
                {cta.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Dismiss button */}
      {onDismiss && (
        <button
          onClick={onDismiss}
          className="flex-shrink-0 hover:bg-destructive/20 rounded p-1 transition-colors"
          aria-label="Dismiss error"
        >
          <X className="h-4 w-4" />
        </button>
      )}
    </div>
  );
}
