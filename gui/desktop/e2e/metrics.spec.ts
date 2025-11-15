import { test, expect } from '@playwright/test';

test.describe('Metrics Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });
  });

  test('should display metrics panel and handle WS updates', async ({ page }) => {
    // Start replay with boot_with_metrics sample
    const replaySection = page.locator('text=/Replay/i').first();
    await replaySection.waitFor({ state: 'visible' });

    // Find and click sample selector
    const sampleSelect = page.locator('select').filter({ hasText: /boot_with_metrics/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_with_metrics');
    }

    // Start replay
    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Wait for boot markers to appear
    await expect(page.getByText(/INIT_STARTED|BOOT_COMPLETE/i)).toBeVisible({ timeout: 30000 });

    // Verify terminal has output
    const terminal = page.locator('.xterm-screen');
    await expect(terminal).toBeVisible();

    // Check that metrics panel shows metrics
    const metricsPanel = page.locator('text=/Metrics/i').first();
    await metricsPanel.waitFor({ state: 'visible' });

    // Wait for at least one metric series to appear
    await page.waitForSelector('text=/nn_infer_us|irq_latency_ns|memory_alloc_ns/i', { timeout: 30000 });

    // Verify chart updates from WebSocket (check for data points)
    const chartContainer = page.locator('text=/nn_infer_us/i').locator('..');
    await expect(chartContainer).toBeVisible();

    // Test pause/resume
    const pauseButton = page.getByRole('button', { name: /pause/i });
    if (await pauseButton.count() > 0) {
      await pauseButton.click();
      await expect(page.getByRole('button', { name: /resume/i })).toBeVisible();

      // Resume
      await page.getByRole('button', { name: /resume/i }).click();
      await expect(pauseButton).toBeVisible();
    }
  });

  test('should export metrics data', async ({ page }) => {
    // Wait for metrics to be available
    await page.waitForSelector('text=/nn_infer_us|irq_latency_ns/i', { timeout: 30000 });

    // Click on a metric series to select it
    const metricRow = page.locator('text=/nn_infer_us/i').first();
    await metricRow.click();

    // Wait for export buttons to appear
    await page.waitForSelector('button:has-text("CSV")', { timeout: 5000 });

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Click CSV export
    await page.getByRole('button', { name: 'CSV' }).first().click();

    // Verify download started
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/\.csv$/);

    // Test JSON export
    const jsonDownloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: 'JSON' }).first().click();

    const jsonDownload = await jsonDownloadPromise;
    expect(jsonDownload.suggestedFilename()).toMatch(/\.json$/);
  });

  test('should display dashboard cards with correct states', async ({ page }) => {
    // Check for dashboard status cards
    await expect(page.getByText('QEMU')).toBeVisible();
    await expect(page.getByText('Shell')).toBeVisible();
    await expect(page.getByText('Replay')).toBeVisible();
    await expect(page.getByText('Autonomy')).toBeVisible();

    // Verify initial states
    await expect(page.locator('text=/idle|not ready|disabled/i')).toHaveCount(4);
  });

  test('should show default metric charts on dashboard', async ({ page }) => {
    // Start QEMU or replay to generate metrics
    const startButton = page.getByRole('button', { name: /start.*replay/i });
    if (await startButton.count() > 0) {
      await startButton.click();

      // Wait for metrics to appear
      await page.waitForTimeout(5000);

      // Check for default chart labels
      await expect(page.getByText(/NN Inference/i)).toBeVisible();
      await expect(page.getByText(/IRQ Latency/i)).toBeVisible();
      await expect(page.getByText(/Memory Alloc/i)).toBeVisible();
      await expect(page.getByText(/Context Switch/i)).toBeVisible();
    }
  });
});
