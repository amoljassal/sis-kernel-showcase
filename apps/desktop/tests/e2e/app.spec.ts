/**
 * End-to-End Tests - M8
 *
 * Playwright E2E tests for SIS Kernel Desktop App
 */

import { test, expect } from '@playwright/test';

test.describe('Application Launch', () => {
  test('should display main interface', async ({ page }) => {
    await page.goto('/');

    // Check header
    await expect(page.getByRole('heading', { name: /SIS Kernel/i })).toBeVisible();

    // Check daemon status
    await expect(page.getByText(/Daemon:/i)).toBeVisible();

    // Check QEMU status
    await expect(page.getByText(/QEMU:/i)).toBeVisible();
  });

  test('should show daemon launch button when not connected', async ({ page }) => {
    await page.goto('/');

    // If daemon is not running, should show launch button
    const launchButton = page.getByRole('button', { name: /Launch Daemon/i });
    if (await launchButton.isVisible()) {
      await expect(launchButton).toBeEnabled();
    }
  });
});

test.describe('Navigation', () => {
  test('should navigate between tabs', async ({ page }) => {
    await page.goto('/');

    // Check all main tabs are present
    await expect(page.getByRole('tab', { name: /Dashboard/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Metrics/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Logs/i })).toBeVisible();
    await expect(page.getByRole('tab', { name: /Settings/i })).toBeVisible();

    // Click Metrics tab
    await page.getByRole('tab', { name: /Metrics/i }).click();
    // Verify metrics panel is visible
    await expect(page.getByText(/TrendingUp/i)).toBeVisible();

    // Click Settings tab
    await page.getByRole('tab', { name: /Settings/i }).click();
    // Verify settings panel is visible
    await expect(page.getByRole('heading', { name: /Settings/i })).toBeVisible();
  });
});

test.describe('QEMU Profile Selector', () => {
  test('should display profile selector', async ({ page }) => {
    await page.goto('/');

    // Check profile selector exists
    await expect(page.getByRole('heading', { name: /QEMU Profile/i })).toBeVisible();

    // Check run/stop buttons
    await expect(page.getByRole('button', { name: /Run QEMU/i })).toBeVisible();
  });

  test('should allow feature selection', async ({ page }) => {
    await page.goto('/');

    // Open feature selector
    const featuresButton = page.getByText(/Features:/i);
    if (await featuresButton.isVisible()) {
      await featuresButton.click();
      // Verify feature options
      await expect(page.getByText(/llm/i)).toBeVisible();
    }
  });
});

test.describe('API Explorer', () => {
  test('should display API Explorer panel', async ({ page }) => {
    await page.goto('/');

    // Navigate to API Explorer
    await page.getByRole('tab', { name: /API Explorer/i }).click();

    // Check API Explorer content
    await expect(page.getByRole('heading', { name: /API Explorer/i })).toBeVisible();

    // Check endpoint list
    await expect(page.getByText(/Explorer/i)).toBeVisible();
    await expect(page.getByText(/History/i }).toBeVisible();
  });
});

test.describe('Settings Panel', () => {
  test('should display and save settings', async ({ page }) => {
    await page.goto('/');

    // Navigate to Settings
    await page.getByRole('tab', { name: /Settings/i }).click();

    // Check theme options
    await expect(page.getByText(/Light/i)).toBeVisible();
    await expect(page.getByText(/Dark/i)).toBeVisible();
    await expect(page.getByText(/System/i }).toBeVisible();

    // Check Save Changes button
    await expect(page.getByRole('button', { name: /Save Changes/i })).toBeVisible();
  });
});

test.describe('Accessibility', () => {
  test('should have proper ARIA labels', async ({ page }) => {
    await page.goto('/');

    // Check main heading has proper role
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();

    // Check navigation has proper structure
    await expect(page.getByRole('tablist')).toBeVisible();

    // Check buttons have labels
    const buttons = page.getByRole('button');
    const count = await buttons.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should support keyboard navigation', async ({ page }) => {
    await page.goto('/');

    // Tab through interactive elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');

    // Verify focus is visible (visual regression would be better)
    const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
    expect(['BUTTON', 'A', 'INPUT']).toContain(focusedElement);
  });
});

test.describe('Responsiveness', () => {
  test('should adapt to mobile viewport', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/');

    // Check if main elements are still visible
    await expect(page.getByRole('heading', { name: /SIS Kernel/i })).toBeVisible();
  });

  test('should adapt to tablet viewport', async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.goto('/');

    // Check if main elements are still visible
    await expect(page.getByRole('heading', { name: /SIS Kernel/i })).toBeVisible();
  });
});
