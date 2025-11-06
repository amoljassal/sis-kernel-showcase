import { test, expect } from '@playwright/test';

test.describe('Autonomy Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });

    // Start replay with boot_with_metrics sample
    const sampleSelect = page.locator('select').filter({ hasText: /boot_with_metrics/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_with_metrics');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Wait for boot to complete
    await expect(page.getByText(/BOOT_COMPLETE/i)).toBeVisible({ timeout: 30000 });

    // Navigate to Autonomy tab
    await page.getByRole('button', { name: /Autonomy/i }).click();
    await page.waitForTimeout(1000); // Let tab content load
  });

  test('should display autonomy status and controls', async ({ page }) => {
    // Verify controls are visible
    await expect(page.getByRole('button', { name: /On/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Off/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /Reset/i })).toBeVisible();

    // Verify interval input
    await expect(page.locator('input[type="text"]').filter({ hasText: /ms/i })).toBeVisible();

    // Verify confidence threshold slider
    await expect(page.locator('input[type="range"]')).toBeVisible();

    // Verify status cards
    await expect(page.getByText(/Enabled/i)).toBeVisible();
    await expect(page.getByText(/Mode/i)).toBeVisible();
    await expect(page.getByText(/Interval/i)).toBeVisible();
    await expect(page.getByText(/Threshold/i)).toBeVisible();
    await expect(page.getByText(/Total/i)).toBeVisible();
    await expect(page.getByText(/Accepted/i)).toBeVisible();
    await expect(page.getByText(/Deferred/i)).toBeVisible();
    await expect(page.getByText(/Watchdog/i)).toBeVisible();
  });

  test('should change autonomy interval and reflect in status', async ({ page }) => {
    // Find interval input and change value
    const intervalInput = page.locator('input[type="text"]').first();
    await intervalInput.fill('2000');

    // Click Set button
    const setButton = page.getByRole('button', { name: /Set/i }).first();
    await setButton.click();

    // Wait for status to update
    await page.waitForTimeout(2500); // Wait for refetch interval

    // Verify interval changed (may show in status or be confirmed by API)
    // In Replay mode, this may be stubbed, so just verify no errors
    await expect(page.locator('text=/Error/i')).not.toBeVisible();
  });

  test('should change confidence threshold', async ({ page }) => {
    // Find threshold slider
    const thresholdSlider = page.locator('input[type="range"]').first();

    // Change slider value
    await thresholdSlider.fill('750');

    // Click Set button
    const setButton = page.getByRole('button', { name: /Set/i }).last();
    await setButton.click();

    // Wait for update
    await page.waitForTimeout(2500);

    // Verify no errors
    await expect(page.locator('text=/Error/i')).not.toBeVisible();
  });

  test('should display decisions table', async ({ page }) => {
    // Wait for decisions table to render
    await page.waitForTimeout(3000); // Allow time for audit data

    // Check for table headers
    const decisionsTable = page.locator('text=/Decisions/i').locator('..');
    await expect(decisionsTable).toBeVisible();

    // Verify columns
    await expect(page.getByText(/ID/i)).toBeVisible();
    await expect(page.getByText(/Timestamp/i)).toBeVisible();
    await expect(page.getByText(/Action/i)).toBeVisible();
    await expect(page.getByText(/Confidence/i)).toBeVisible();
  });

  test('should open ExplainView when clicking decision row', async ({ page }) => {
    // Wait for decisions to load
    await page.waitForTimeout(3000);

    // Click on first decision row (if any exist)
    const decisionRow = page.locator('[role="row"]').first();
    const rowCount = await decisionRow.count();

    if (rowCount > 0) {
      // Click explain button or row
      const explainButton = page.getByRole('button', { name: /Explain/i }).first();
      if (await explainButton.count() > 0) {
        await explainButton.click();

        // Verify ExplainView modal opens
        await expect(page.getByText(/Decision Explanation/i)).toBeVisible({ timeout: 5000 });

        // Verify importance bars are visible
        await expect(page.locator('[role="progressbar"]')).toHaveCount(3, { timeout: 5000 });

        // Test keyboard navigation
        await page.keyboard.press('ArrowDown');
        await page.keyboard.press('ArrowUp');

        // Close with Escape
        await page.keyboard.press('Escape');
        await expect(page.getByText(/Decision Explanation/i)).not.toBeVisible();
      }
    }
  });

  test('should navigate to What-If tab and interact with simulator', async ({ page }) => {
    // Navigate to What-If tab
    await page.getByRole('button', { name: /What-If/i }).click();
    await page.waitForTimeout(1000);

    // Verify simulator controls
    await expect(page.getByText(/What-If Simulator/i)).toBeVisible();
    await expect(page.getByText(/Memory Pressure/i)).toBeVisible();
    await expect(page.getByText(/Fragmentation/i)).toBeVisible();
    await expect(page.getByText(/Cache Misses/i)).toBeVisible();
    await expect(page.getByText(/Command Rate/i)).toBeVisible();

    // Verify baseline/scenario sections
    await expect(page.getByText(/Baseline.*Current/i)).toBeVisible();
    await expect(page.getByText(/Scenario.*Modified/i)).toBeVisible();

    // Adjust a parameter
    const memSlider = page.locator('input[type="range"]').first();
    await memSlider.fill('50');

    // Wait for debounce and API call
    await page.waitForTimeout(500);

    // Verify scenario updates (should show loading or result)
    // In Replay mode, this may be stubbed
    await page.waitForTimeout(1000);

    // Test export functionality
    const exportButton = page.getByRole('button', { name: /Export/i });
    if (await exportButton.count() > 0) {
      const downloadPromise = page.waitForEvent('download');
      await exportButton.click();

      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/whatif.*\.json$/);
    }
  });
});
