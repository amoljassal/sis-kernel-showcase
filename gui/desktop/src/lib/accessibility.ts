/**
 * Accessibility utilities - M7 Polish & UX
 *
 * Helper functions for ARIA attributes and keyboard navigation
 */

/**
 * Generate ARIA label for metric values
 */
export function getMetricAriaLabel(name: string, value: number, unit?: string): string {
  const unitText = unit || '';
  return `${name}: ${value.toFixed(2)} ${unitText}`.trim();
}

/**
 * Generate ARIA label for status badge
 */
export function getStatusAriaLabel(status: string): string {
  const statusMap: Record<string, string> = {
    running: 'QEMU is running',
    idle: 'QEMU is idle',
    starting: 'QEMU is starting',
    stopping: 'QEMU is stopping',
    error: 'QEMU encountered an error',
  };
  return statusMap[status.toLowerCase()] || `Status: ${status}`;
}

/**
 * Keyboard navigation handler for focusable elements
 */
export function handleKeyboardNav(
  event: React.KeyboardEvent,
  handlers: {
    onEnter?: () => void;
    onSpace?: () => void;
    onEscape?: () => void;
    onArrowUp?: () => void;
    onArrowDown?: () => void;
  }
) {
  switch (event.key) {
    case 'Enter':
      if (handlers.onEnter) {
        event.preventDefault();
        handlers.onEnter();
      }
      break;
    case ' ':
      if (handlers.onSpace) {
        event.preventDefault();
        handlers.onSpace();
      }
      break;
    case 'Escape':
      if (handlers.onEscape) {
        event.preventDefault();
        handlers.onEscape();
      }
      break;
    case 'ArrowUp':
      if (handlers.onArrowUp) {
        event.preventDefault();
        handlers.onArrowUp();
      }
      break;
    case 'ArrowDown':
      if (handlers.onArrowDown) {
        event.preventDefault();
        handlers.onArrowDown();
      }
      break;
  }
}

/**
 * Focus trap for modals
 */
export function trapFocus(element: HTMLElement) {
  const focusableSelectors = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
  const focusableElements = element.querySelectorAll<HTMLElement>(focusableSelectors);
  const firstFocusable = focusableElements[0];
  const lastFocusable = focusableElements[focusableElements.length - 1];

  function handleTab(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;

    if (event.shiftKey) {
      if (document.activeElement === firstFocusable) {
        event.preventDefault();
        lastFocusable.focus();
      }
    } else {
      if (document.activeElement === lastFocusable) {
        event.preventDefault();
        firstFocusable.focus();
      }
    }
  }

  element.addEventListener('keydown', handleTab);

  // Focus first element
  firstFocusable?.focus();

  return () => {
    element.removeEventListener('keydown', handleTab);
  };
}

/**
 * Announce screen reader message
 */
export function announceToScreenReader(message: string, priority: 'polite' | 'assertive' = 'polite') {
  const announcement = document.createElement('div');
  announcement.setAttribute('role', 'status');
  announcement.setAttribute('aria-live', priority);
  announcement.setAttribute('aria-atomic', 'true');
  announcement.className = 'sr-only';
  announcement.textContent = message;

  document.body.appendChild(announcement);

  setTimeout(() => {
    document.body.removeChild(announcement);
  }, 1000);
}
