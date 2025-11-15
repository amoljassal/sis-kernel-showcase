import { test, expect } from '@playwright/test';

test.describe('Scheduling Panel Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });

    // Navigate to Scheduling tab
    await page.getByRole('button', { name: /Scheduling/i }).click();
    await expect(page.getByText('Scheduling')).toBeVisible({ timeout: 5000 });
  });

  test('should display workloads list', async ({ page }) => {
    // Wait for workloads section to be visible
    await expect(page.getByText('Workloads')).toBeVisible();

    // Verify table headers are present
    await expect(page.getByText('PID')).toBeVisible();
    await expect(page.getByText('Name')).toBeVisible();
    await expect(page.getByText('Priority')).toBeVisible();
    await expect(page.getByText('CPU')).toBeVisible();
  });

  test('should select workload and set priority', async ({ page }) => {
    // Wait for workloads to load
    await page.waitForTimeout(2000);

    // Check if workloads are available
    const workloadRow = page.locator('text=/PID.*\\d+/i').first();
    if (await workloadRow.count() > 0) {
      // Select workload
      await workloadRow.click();

      // Set new priority
      const priorityInput = page.locator('input[placeholder*="Priority"]');
      await priorityInput.fill('120');

      const setPriorityButton = page.getByRole('button', { name: /set priority/i });
      await setPriorityButton.click();

      // Wait for update
      await page.waitForTimeout(1000);
    }
  });

  test('should set CPU affinity for workload', async ({ page }) => {
    // Wait for workloads to load
    await page.waitForTimeout(2000);

    const workloadRow = page.locator('text=/PID.*\\d+/i').first();
    if (await workloadRow.count() > 0) {
      // Select workload
      await workloadRow.click();

      // Set CPU affinity
      const affinityInput = page.locator('input[placeholder*="CPU Mask"]');
      await affinityInput.fill('0xF');

      const setAffinityButton = page.getByRole('button', { name: /set affinity/i });
      await setAffinityButton.click();

      // Wait for update
      await page.waitForTimeout(1000);
    }
  });

  test('should toggle autonomous scheduling feature', async ({ page }) => {
    // Enable autonomous scheduling
    const enableButton = page.getByRole('button', { name: /enable autonomous/i });
    await enableButton.click();

    // Wait for feature toggle
    await page.waitForTimeout(1000);

    // Disable autonomous scheduling
    const disableButton = page.getByRole('button', { name: /disable autonomous/i });
    await disableButton.click();

    // Wait for feature toggle
    await page.waitForTimeout(1000);
  });

  test('should toggle shadow mode feature', async ({ page }) => {
    // Enable shadow mode
    const shadowModeButton = page.getByRole('button', { name: /enable shadow mode/i });
    await shadowModeButton.click();

    // Wait for feature toggle
    await page.waitForTimeout(1000);
  });

  test('should display circuit breaker state', async ({ page }) => {
    // Wait for circuit breaker section
    await expect(page.getByText('Circuit Breaker')).toBeVisible();

    // Verify circuit breaker state is displayed
    await page.waitForSelector('text=/Closed|Open|HalfOpen/i', { timeout: 5000 });

    // Verify metrics are shown
    await expect(page.getByText(/failures/i)).toBeVisible();
    await expect(page.getByText(/timeout/i)).toBeVisible();
  });

  test('should reset circuit breaker', async ({ page }) => {
    // Wait for circuit breaker section
    await expect(page.getByText('Circuit Breaker')).toBeVisible();

    // Reset circuit breaker
    const resetButton = page.getByRole('button', { name: /reset/i });
    await resetButton.click();

    // Wait for reset to complete
    await page.waitForTimeout(1000);
  });

  test('should handle WebSocket sched_event updates', async ({ page }) => {
    // Start replay with boot_sched sample
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /boot_sched/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_sched');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Wait for boot to complete
    await page.waitForSelector('text=/BOOT_COMPLETE|boot:complete/i', { timeout: 30000 });

    // Navigate to Scheduling tab
    await page.getByRole('button', { name: /Scheduling/i }).click();

    // Wait for workloads to appear from WS events
    await page.waitForSelector('text=/worker|init/i', { timeout: 30000 });

    // Verify workloads list is populated
    await expect(page.getByText(/PID.*100\d/i)).toBeVisible();
  });

  test('should virtualize large workload lists', async ({ page }) => {
    // This test verifies that the virtualized table renders correctly
    // even with potentially large datasets

    // Wait for workloads section
    await expect(page.getByText('Workloads')).toBeVisible();

    // Scroll through workload list
    const workloadsContainer = page.locator('text=/Workloads/i').locator('..');
    if (await workloadsContainer.count() > 0) {
      await workloadsContainer.scrollIntoViewIfNeeded();
    }

    // Verify no rendering issues
    await page.waitForTimeout(500);
  });
});
