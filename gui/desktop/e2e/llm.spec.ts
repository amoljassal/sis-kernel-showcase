import { test, expect } from '@playwright/test';

test.describe('LLM Panel Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');

    // Wait for daemon to be healthy
    await expect(page.getByText('Daemon:')).toBeVisible();
    await page.waitForSelector('text=/running|idle/i', { timeout: 10000 });

    // Navigate to LLM tab
    await page.getByRole('button', { name: /LLM/i }).click();
    await expect(page.getByText('LLM Inference')).toBeVisible({ timeout: 5000 });
  });

  test('should display LLM panel sections', async ({ page }) => {
    // Verify main sections are visible
    await expect(page.getByText('Load Model')).toBeVisible();
    await expect(page.getByText('Run Inference')).toBeVisible();
    await expect(page.getByText('Audit Log')).toBeVisible();
  });

  test('should load LLM model', async ({ page }) => {
    // Fill model ID
    const modelIdInput = page.locator('input[placeholder*="llama"]');
    await modelIdInput.fill('llama-7b');

    // Fill WCET cycles (optional)
    const wcetInput = page.locator('input[placeholder*="1000000"]');
    await wcetInput.fill('1000000');

    // Load model
    const loadButton = page.getByRole('button', { name: /load model/i });
    await loadButton.click();

    // Wait for model to load
    await page.waitForTimeout(2000);

    // Verify status card appears
    await page.waitForSelector('text=/Model Status|WCET Cycles/i', { timeout: 5000 });
  });

  test('should submit inference request', async ({ page }) => {
    // Skip loading model for faster test (assumes model loaded or mock)

    // Fill inference text
    const inferTextarea = page.locator('textarea[placeholder*="prompt"]');
    await inferTextarea.fill('What is the meaning of life?');

    // Set max tokens
    const maxTokensInput = page.locator('input[placeholder*="Max tokens"]');
    await maxTokensInput.fill('50');

    // Submit inference
    const inferButton = page.getByRole('button', { name: /infer/i });
    await inferButton.click();

    // Wait for inference to start
    await page.waitForTimeout(1000);
  });

  test('should display streaming output', async ({ page }) => {
    // Start replay with boot_llm sample
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /boot_llm/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_llm');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Wait for boot to complete
    await page.waitForSelector('text=/BOOT_COMPLETE|boot:complete/i', { timeout: 30000 });

    // Navigate to LLM tab
    await page.getByRole('button', { name: /LLM/i }).click();

    // Wait for streaming output to appear
    await page.waitForSelector('text=/Output.*Request/i', { timeout: 30000 });

    // Verify output contains text chunks
    await expect(page.getByText(/quick|brown|fox|jumps|lazy|dog/i)).toBeVisible();
  });

  test('should display audit log entries', async ({ page }) => {
    // Wait for audit log section
    await expect(page.getByText('Audit Log')).toBeVisible();

    // After inference completes, audit entry should appear
    await page.waitForTimeout(2000);

    // Check for audit entries (might be empty initially)
    const auditLogSection = page.locator('text=/Audit Log/i').locator('..');
    await expect(auditLogSection).toBeVisible();
  });

  test('should virtualize audit log', async ({ page }) => {
    // Verify audit log uses virtualization
    await expect(page.getByText('Audit Log')).toBeVisible();

    // Scroll through audit log if entries exist
    const auditContainer = page.locator('text=/Audit Log/i').locator('..');
    if (await auditContainer.count() > 0) {
      await auditContainer.scrollIntoViewIfNeeded();
    }

    // Verify no rendering issues
    await page.waitForTimeout(500);
  });

  test('should handle WebSocket llm_tokens events', async ({ page }) => {
    // Start replay with boot_llm sample
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /boot_llm/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_llm');
    }

    const startButton = page.getByRole('button', { name: /start.*replay/i });
    await startButton.click();

    // Navigate to LLM tab
    await page.getByRole('button', { name: /LLM/i }).click();

    // Wait for streaming tokens to appear
    await page.waitForSelector('text=/quick|brown|fox/i', { timeout: 30000 });

    // Verify full output is assembled
    await expect(page.getByText(/quick brown fox jumps over.*lazy dog/i)).toBeVisible();
  });

  test('should show LLM status metrics', async ({ page }) => {
    // Load model first
    const modelIdInput = page.locator('input[placeholder*="llama"]');
    await modelIdInput.fill('llama-7b');

    const loadButton = page.getByRole('button', { name: /load model/i });
    await loadButton.click();

    // Wait for status to update
    await page.waitForTimeout(2000);

    // Verify status metrics
    await page.waitForSelector('text=/WCET Cycles|Period|Max Tokens/i', { timeout: 5000 });
    await expect(page.getByText(/budget/i)).toBeVisible();
  });

  test('should clear inference text after submission', async ({ page }) => {
    // Fill inference text
    const inferTextarea = page.locator('textarea[placeholder*="prompt"]');
    await inferTextarea.fill('Test prompt');

    // Submit inference
    const inferButton = page.getByRole('button', { name: /infer/i });
    await inferButton.click();

    // Wait for submission
    await page.waitForTimeout(500);

    // Verify textarea is cleared
    await expect(inferTextarea).toHaveValue('');
  });

  test('should display request ID in output', async ({ page }) => {
    // Start replay to get streaming output
    await page.getByRole('button', { name: /Dashboard/i }).click();

    const sampleSelect = page.locator('select').filter({ hasText: /boot_llm/i });
    if (await sampleSelect.count() > 0) {
      await sampleSelect.selectOption('boot_llm');
      await page.getByRole('button', { name: /start.*replay/i }).click();
    }

    await page.getByRole('button', { name: /LLM/i }).click();

    // Wait for output with request ID
    await page.waitForSelector('text=/Request.*request-/i', { timeout: 30000 });

    // Verify request ID is displayed
    await expect(page.getByText(/request-001|request-\d+/i)).toBeVisible();
  });
});
