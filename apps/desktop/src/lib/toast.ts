/**
 * Simple toast notification system
 */

type ToastType = 'success' | 'error' | 'info';

interface Toast {
  id: string;
  message: string;
  type: ToastType;
}

let toastContainer: HTMLDivElement | null = null;
let toastIdCounter = 0;

function ensureContainer() {
  if (!toastContainer) {
    toastContainer = document.createElement('div');
    toastContainer.id = 'toast-container';
    toastContainer.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      z-index: 9999;
      display: flex;
      flex-direction: column;
      gap: 8px;
      pointer-events: none;
    `;
    document.body.appendChild(toastContainer);
  }
  return toastContainer;
}

function getToastStyles(type: ToastType): string {
  const base = `
    padding: 12px 16px;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    font-size: 14px;
    font-weight: 500;
    pointer-events: auto;
    animation: slideIn 0.3s ease-out;
  `;

  const colors = {
    success: 'background: #10b981; color: white;',
    error: 'background: #ef4444; color: white;',
    info: 'background: #3b82f6; color: white;',
  };

  return base + colors[type];
}

function showToast(message: string, type: ToastType = 'info', duration = 3000) {
  const container = ensureContainer();
  const id = `toast-${toastIdCounter++}`;

  const toastEl = document.createElement('div');
  toastEl.id = id;
  toastEl.textContent = message;
  toastEl.style.cssText = getToastStyles(type);
  toastEl.setAttribute('role', 'status');
  toastEl.setAttribute('aria-live', 'polite');

  container.appendChild(toastEl);

  setTimeout(() => {
    toastEl.style.animation = 'slideOut 0.3s ease-in';
    setTimeout(() => {
      toastEl.remove();
    }, 300);
  }, duration);
}

export function showSuccessToast(message: string, duration?: number) {
  showToast(message, 'success', duration);
}

export function showErrorToast(message: string, duration?: number) {
  showToast(message, 'error', duration);
}

export function showInfoToast(message: string, duration?: number) {
  showToast(message, 'info', duration);
}

// Add CSS animation
const style = document.createElement('style');
style.textContent = `
  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  @keyframes slideOut {
    from {
      transform: translateX(0);
      opacity: 1;
    }
    to {
      transform: translateX(100%);
      opacity: 0;
    }
  }
`;
document.head.appendChild(style);
