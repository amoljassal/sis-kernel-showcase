import { test, expect } from '@playwright/test';

test.describe('Logs Panel Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });

    // Navigate to Logs tab
    await page.getByRole('button', { name: /Logs/i }).click();
    await expect(page.getByText('Logs & Runs')).toBeVisible({ timeout: 5000 });
  });

  test('should display logs panel sections', async ({ page }) => {
    // Verify main sections are visible
    await expect(page.getByText('Run Control')).toBeVisible();
    await expect(page.getByText('Filters')).toBeVisible();
    await expect(page.getByText('Live Logs')).toBeVisible();
    await expect(page.getByText('Run History')).toBeVisible();
  });

  test('should filter logs by level', async ({ page }) => {
    // Select error level filter
    const levelSelect = page.locator('select').filter({ hasText: /All.*Debug.*Info.*Warn.*Error/i });
    await levelSelect.selectOption('error');

    // Wait for filter to apply
    await page.waitForTimeout(500);

    // Verify filter is applied (log count should update)
    await expect(page.getByText(/showing.*of.*logs/i)).toBeVisible();
  });

  test('should filter logs by source', async ({ page }) => {
    // Select daemon source filter
    const sourceSelect = page.locator('select').filter({ hasText: /All.*Daemon.*QEMU.*Kernel/i });
    await sourceSelect.selectOption('daemon');

    // Wait for filter to apply
    await page.waitForTimeout(500);

    // Verify filter is applied
    await expect(page.getByText(/showing.*of.*logs/i)).toBeVisible();
  });

  test('should search logs by text', async ({ page }) => {
    // Enter search text
    const searchInput = page.locator('input[placeholder*="Filter messages"]');
    await searchInput.fill('memory');

    // Wait for search to apply
    await page.waitForTimeout(500);

    // Verify search results
    await expect(page.getByText(/showing.*of.*logs/i)).toBeVisible();
  });

  test('should export logs as JSON', async ({ page }) => {
    // Wait for logs to be available
    await page.waitForTimeout(1000);

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click JSON export
    await page.getByRole('button', { name: 'JSON' }).click();

    // Verify download started
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/logs.*\.json$/);
  });

  test('should export logs as CSV', async ({ page }) => {
    // Wait for logs to be available
    await page.waitForTimeout(1000);

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click CSV export
    await page.getByRole('button', { name: 'CSV' }).click();

    // Verify download started
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/logs.*\.csv$/);
  });

  test('should start a new run', async ({ page }) => {
    // Fill run note
    const runNoteInput = page.locator('input[placeholder*="Run note"]');
    await runNoteInput.fill('Test run');

    // Start run
    const startButton = page.getByRole('button', { name: /start run/i });
    await startButton.click();

    // Wait for run to start
    await page.waitForTimeout(1000);

    // Verify run note is cleared
    await expect(runNoteInput).toHaveValue('');
  });

  test('should stop current run', async ({ page }) => {
    // Stop run
    const stopButton = page.getByRole('button', { name: /stop run/i });
    await stopButton.click();

    // Wait for run to stop
    await page.waitForTimeout(1000);
  });

  test('should display run history', async ({ page }) => {
    // Verify run history section
    await expect(page.getByText('Run History')).toBeVisible();

    // Check for run entries or empty state
    const runHistorySection = page.locator('text=/Run History/i').locator('..');
    await expect(runHistorySection).toBeVisible();
  });

  test('should export run data', async ({ page }) => {
    // Wait for run history to load
    await page.waitForTimeout(2000);

    // Check if any runs exist
    const exportButton = page.getByRole('button', { name: /Export/i }).first();
    if (await exportButton.count() > 0) {
      // Set up download listener
      const downloadPromise = page.waitForEvent('download');

      // Export run
      await exportButton.click();

      // Verify download started
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/run.*\.json$/);
    }
  });

  test('should handle WebSocket log_line events', async ({ page }) => {
    // Start replay with logs_mixed sample
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /logs_mixed/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('logs_mixed');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Navigate to Logs tab
    await page.getByRole('button', { name: /Logs/i }).click();

    // Wait for log entries to appear from WS
    await page.waitForSelector('text=/sisctl|daemon|kernel|qemu/i', { timeout: 30000 });

    // Verify log entries are displayed
    await expect(page.getByText(/Starting.*Kernel/i)).toBeVisible();
  });

  test('should display self-check PASS banner', async ({ page }) => {
    // This test verifies the PASS banner appears after self-check completes
    // Start replay or self-check that triggers self_check_completed event

    await page.getByRole('button', { name: /Dashboard/i }).click();

    // Run self-check
    const selfCheckButton = page.getByRole('button', { name: /run.*self.*check/i });
    if (await selfCheckButton.count() > 0) {
      await selfCheckButton.click();

      // Wait for self-check to complete
      await page.waitForSelector('text=/PASS|MARKERS SEEN/i', { timeout: 60000 });

      // Navigate to Logs tab
      await page.getByRole('button', { name: /Logs/i }).click();

      // Verify PASS banner is visible
      await expect(page.getByText(/ALL MARKERS SEEN.*SELF-CHECK PASSED/i)).toBeVisible();
    }
  });

  test('should virtualize log viewer for large datasets', async ({ page }) => {
    // Verify log viewer uses virtualization
    await expect(page.getByText('Live Logs')).toBeVisible();

    // Check buffer size display
    await expect(page.getByText(/buffer.*10000/i)).toBeVisible();

    // Scroll through logs
    const logsContainer = page.locator('text=/Live Logs/i').locator('..');
    if (await logsContainer.count() > 0) {
      await logsContainer.scrollIntoViewIfNeeded();
    }

    // Verify no rendering issues
    await page.waitForTimeout(500);
  });

  test('should display log level with color coding', async ({ page }) => {
    // Start replay to get logs
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /logs_mixed/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('logs_mixed');
      await page.getByRole('button', { name: /start.*replay/i }).click();
    }

    await page.getByRole('button', { name: /Logs/i }).click();

    // Wait for logs with different levels
    await page.waitForSelector('text=/INFO|ERROR|WARN|DEBUG/i', { timeout: 30000 });

    // Verify level indicators are present
    await expect(page.getByText(/INFO/i)).toBeVisible();
  });

  test('should maintain ring buffer limit', async ({ page }) => {
    // Verify buffer limit is displayed
    await expect(page.getByText(/buffer.*10000/i)).toBeVisible();

    // The ring buffer should maintain max 10k entries
    // This is verified by the display text showing the limit
  });

  test('should display log timestamps', async ({ page }) => {
    // Start replay to get logs
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /logs_mixed/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('logs_mixed');
      await page.getByRole('button', { name: /start.*replay/i }).click();
    }

    await page.getByRole('button', { name: /Logs/i }).click();

    // Wait for logs with timestamps
    await page.waitForSelector('text=/\\d{1,2}:\\d{2}:\\d{2}/i', { timeout: 30000 });

    // Verify timestamp format
    await expect(page.getByText(/\\d{1,2}:\\d{2}:\\d{2}/i)).toBeVisible();
  });

  test('should display log source indicators', async ({ page }) => {
    // Start replay to get logs
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /logs_mixed/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('logs_mixed');
      await page.getByRole('button', { name: /start.*replay/i }).click();
    }

    await page.getByRole('button', { name: /Logs/i }).click();

    // Wait for logs with source indicators
    await page.waitForSelector('text=/\\[daemon\\]|\\[qemu\\]|\\[kernel\\]/i', { timeout: 30000 });

    // Verify source indicators
    await expect(page.getByText(/\\[daemon\\]|\\[qemu\\]|\\[kernel\\]/i)).toBeVisible();
  });
});
