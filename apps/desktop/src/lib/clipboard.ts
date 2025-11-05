/**
 * Clipboard utilities with toast feedback
 */

import { showSuccessToast, showErrorToast } from './toast';

export async function copyToClipboard(text: string, successMessage = 'Copied to clipboard'): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    showSuccessToast(successMessage);
    return true;
  } catch (err) {
    console.error('Failed to copy to clipboard:', err);
    showErrorToast('Failed to copy to clipboard');
    return false;
  }
}

export async function copyJSONToClipboard(data: any, successMessage = 'JSON copied to clipboard'): Promise<boolean> {
  try {
    const json = JSON.stringify(data, null, 2);
    return await copyToClipboard(json, successMessage);
  } catch (err) {
    console.error('Failed to serialize JSON:', err);
    showErrorToast('Failed to serialize JSON');
    return false;
  }
}
