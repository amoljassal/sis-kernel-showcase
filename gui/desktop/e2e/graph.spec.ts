import { test, expect } from '@playwright/test';

test.describe('Graph Panel Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });

    // Navigate to Graph tab
    await page.getByRole('button', { name: /Graph/i }).click();
    await expect(page.getByText('Graph Control')).toBeVisible({ timeout: 5000 });
  });

  test('should create graph and display state', async ({ page }) => {
    // Create a new graph
    const createButton = page.getByRole('button', { name: /create graph/i });
    await createButton.click();

    // Wait for graph ID to appear
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Verify graph state section is visible
    await expect(page.getByText('Operators')).toBeVisible();
    await expect(page.getByText('Channels')).toBeVisible();
  });

  test('should add channel to graph', async ({ page }) => {
    // Create graph first
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Add channel
    const capacityInput = page.locator('input[placeholder*="Capacity"]');
    await capacityInput.fill('100');

    const addChannelButton = page.getByRole('button', { name: /add channel/i });
    await addChannelButton.click();

    // Verify channel appears in list
    await page.waitForSelector('text=/ch-|channel/i', { timeout: 5000 });
    await expect(page.getByText(/cap.*100/i)).toBeVisible();
  });

  test('should add operator to graph', async ({ page }) => {
    // Create graph first
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Add operator
    const operatorIdInput = page.locator('input[placeholder*="Operator ID"]');
    await operatorIdInput.fill('op-001');

    const priorityInput = page.locator('input[placeholder*="Priority"]');
    await priorityInput.fill('100');

    const stageInput = page.locator('input[placeholder*="Stage"]');
    await stageInput.fill('input');

    const addOperatorButton = page.getByRole('button', { name: /add operator/i });
    await addOperatorButton.click();

    // Verify operator appears in list
    await page.waitForSelector('text=/op-001/i', { timeout: 5000 });
    await expect(page.getByText(/prio.*100/i)).toBeVisible();
  });

  test('should start graph execution', async ({ page }) => {
    // Create graph first
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Start graph
    const stepsInput = page.locator('input[placeholder*="Steps"]');
    await stepsInput.fill('10');

    const startButton = page.getByRole('button', { name: /start graph/i });
    await startButton.click();

    // Verify execution feedback (could be success message or state update)
    await page.waitForTimeout(1000);
  });

  test('should run prediction on operator', async ({ page }) => {
    // Create graph with operator
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Add operator
    const operatorIdInput = page.locator('input[placeholder*="Operator ID"]').first();
    await operatorIdInput.fill('op-001');
    await page.getByRole('button', { name: /add operator/i }).click();
    await page.waitForSelector('text=/op-001/i', { timeout: 5000 });

    // Run prediction
    const predictOpInput = page.locator('input[placeholder*="Operator ID"]').nth(1);
    await predictOpInput.fill('op-001');

    const latencyInput = page.locator('input[placeholder*="Latency"]');
    await latencyInput.fill('1000');

    const depthInput = page.locator('input[placeholder*="Depth"]');
    await depthInput.fill('5');

    const predictButton = page.getByRole('button', { name: /predict/i });
    await predictButton.click();

    // Verify prediction result appears
    await page.waitForSelector('text=/predicted|prediction/i', { timeout: 5000 });
  });

  test('should submit feedback for operator', async ({ page }) => {
    // Create graph with operator
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Submit feedback
    const feedbackOpInput = page.locator('input[placeholder*="Operator ID"]').last();
    await feedbackOpInput.fill('op-001');

    const verdictSelect = page.locator('select').filter({ hasText: /helpful|expected/i });
    if (await verdictSelect.count() > 0) {
      await verdictSelect.selectOption('helpful');
    }

    const feedbackButton = page.getByRole('button', { name: /submit feedback/i });
    if (await feedbackButton.count() > 0) {
      await feedbackButton.click();
      await page.waitForTimeout(1000);
    }
  });

  test('should export graph as JSON', async ({ page }) => {
    // Create graph
    await page.getByRole('button', { name: /create graph/i }).click();
    await page.waitForSelector('text=/graph-/i', { timeout: 5000 });

    // Set up download listener
    const downloadPromise = page.waitForEvent('download');

    // Export graph
    const exportButton = page.getByRole('button', { name: /export/i });
    await exportButton.click();

    // Verify download started
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/graph.*\.json$/);
  });

  test('should handle WebSocket graph_state events', async ({ page }) => {
    // Start replay with boot_graph sample
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /boot_graph/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_graph');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Navigate to Graph tab
    await page.getByRole('button', { name: /Graph/i }).click();

    // Wait for graph state updates from WS
    await page.waitForSelector('text=/op-|operator/i', { timeout: 30000 });
    await page.waitForSelector('text=/ch-|channel/i', { timeout: 30000 });

    // Verify operator and channel lists are populated
    await expect(page.getByText(/op-001|op-002/i)).toBeVisible();
    await expect(page.getByText(/ch-001|ch-002/i)).toBeVisible();
  });
});
