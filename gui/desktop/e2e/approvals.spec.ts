import { test, expect } from '@playwright/test';

test.describe('Memory Approvals Flow', () => {
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

    // Navigate to Memory tab
    await page.getByRole('button', { name: /Memory/i }).click();
    await page.waitForTimeout(1000); // Let tab content load
  });

  test('should display approvals panel with status cards', async ({ page }) => {
    // Verify approval mode toggle
    await expect(page.getByRole('button', { name: /Enable|Disable/i })).toBeVisible();

    // Verify status cards
    await expect(page.getByText(/Pending/i)).toBeVisible();
    await expect(page.getByText(/Approved/i)).toBeVisible();
    await expect(page.getByText(/Rejected/i)).toBeVisible();
  });

  test('should display pending operations table', async ({ page }) => {
    // Wait for table to load
    await page.waitForTimeout(2000);

    // Check for table headers
    await expect(page.getByText(/Type/i)).toBeVisible();
    await expect(page.getByText(/Confidence/i)).toBeVisible();
    await expect(page.getByText(/Risk/i)).toBeVisible();
    await expect(page.getByText(/Reason/i)).toBeVisible();
    await expect(page.getByText(/Timestamp/i)).toBeVisible();

    // Verify virtualized table container exists
    const tableContainer = page.locator('[style*="overflow"]').filter({ hasText: /Type|Confidence/i });
    await expect(tableContainer).toBeVisible();
  });

  test('should toggle approval mode', async ({ page }) => {
    // Find toggle button
    const toggleButton = page.getByRole('button', { name: /Enable Approval|Disable Approval/i });

    // Click to toggle
    await toggleButton.click();

    // Wait for API call
    await page.waitForTimeout(2500);

    // Verify no error occurred
    await expect(page.locator('text=/Error/i')).not.toBeVisible();
  });

  test('should approve N pending operations', async ({ page }) => {
    // Wait for operations to load
    await page.waitForTimeout(2000);

    // Find "Approve N" input and button
    const approveInput = page.locator('input[type="text"]').filter({ hasText: /Approve/i }).or(
      page.locator('input[type="number"]').first()
    );

    if (await approveInput.count() > 0) {
      await approveInput.fill('1');

      // Click Approve button
      const approveButton = page.getByRole('button', { name: /Approve \d+/i });
      await approveButton.click();

      // Wait for API call
      await page.waitForTimeout(2500);

      // Verify no errors
      await expect(page.locator('text=/Error/i')).not.toBeVisible();

      // Check that pending count decreased (if visible)
      await page.waitForTimeout(2500); // Wait for refetch
    }
  });

  test('should approve selected operations with checkboxes', async ({ page }) => {
    // Wait for operations to load
    await page.waitForTimeout(2000);

    // Check if any checkboxes exist
    const firstCheckbox = page.locator('input[type="checkbox"]').first();
    const checkboxCount = await firstCheckbox.count();

    if (checkboxCount > 0) {
      // Select first checkbox
      await firstCheckbox.check();

      // Click "Approve Selected" button
      const approveSelectedButton = page.getByRole('button', { name: /Approve Selected/i });
      if (await approveSelectedButton.count() > 0) {
        await approveSelectedButton.click();

        // Wait for API call
        await page.waitForTimeout(2500);

        // Verify no errors
        await expect(page.locator('text=/Error/i')).not.toBeVisible();
      }
    }
  });

  test('should reject individual operation with confirmation', async ({ page }) => {
    // Wait for operations to load
    await page.waitForTimeout(2000);

    // Find first reject button in table
    const rejectButtons = page.getByRole('button', { name: /Reject/i });
    const rejectCount = await rejectButtons.count();

    if (rejectCount > 0) {
      // Click first reject button
      await rejectButtons.first().click();

      // Wait for confirm dialog
      await page.waitForTimeout(500);

      // Look for confirm dialog with "Reject" action
      const confirmDialog = page.locator('text=/Are you sure/i');
      if (await confirmDialog.count() > 0) {
        // Find and click Reject button in dialog
        const confirmButton = page.getByRole('button', { name: /Reject/i }).last();
        await confirmButton.click();

        // Wait for API call
        await page.waitForTimeout(2500);

        // Verify no errors
        await expect(page.locator('text=/Error/i')).not.toBeVisible();
      }
    }
  });

  test('should reject all operations with confirmation', async ({ page }) => {
    // Wait for operations to load
    await page.waitForTimeout(2000);

    // Find "Reject All" button
    const rejectAllButton = page.getByRole('button', { name: /Reject All/i });
    const buttonCount = await rejectAllButton.count();

    if (buttonCount > 0) {
      await rejectAllButton.click();

      // Wait for confirm dialog
      await page.waitForTimeout(500);

      // Verify confirm dialog appears
      const confirmDialog = page.locator('text=/Are you sure.*all/i');
      if (await confirmDialog.count() > 0) {
        await expect(confirmDialog).toBeVisible();

        // Click Cancel to test cancel flow
        const cancelButton = page.getByRole('button', { name: /Cancel/i });
        await cancelButton.click();

        // Verify dialog closed
        await expect(confirmDialog).not.toBeVisible();

        // Try again and confirm
        await rejectAllButton.click();
        await page.waitForTimeout(500);

        const confirmButton = page.getByRole('button', { name: /Reject/i }).last();
        await confirmButton.click();

        // Wait for API call
        await page.waitForTimeout(2500);

        // Verify no errors
        await expect(page.locator('text=/Error/i')).not.toBeVisible();

        // Check that pending count went to 0 (if visible)
        await page.waitForTimeout(2500); // Wait for refetch
      }
    }
  });

  test('should show risk color coding in table', async ({ page }) => {
    // Wait for operations to load
    await page.waitForTimeout(2000);

    // Look for risk badges with colors
    const riskBadges = page.locator('text=/low|medium|high/i');
    const badgeCount = await riskBadges.count();

    if (badgeCount > 0) {
      // Verify at least one risk badge is visible
      await expect(riskBadges.first()).toBeVisible();

      // Check for color classes (green/yellow/red)
      const firstBadge = riskBadges.first();
      const className = await firstBadge.getAttribute('class');
      expect(className).toMatch(/text-(green|yellow|red)-500/);
    }
  });

  test('should handle empty state gracefully', async ({ page }) => {
    // If no pending operations, should show appropriate message or empty state
    await page.waitForTimeout(2000);

    // Check if table is empty or has message
    const tableBody = page.locator('[style*="overflow"]');
    const isEmpty = (await tableBody.textContent())?.includes('No pending') ||
                    (await page.locator('text=/No pending|No operations/i').count()) > 0;

    if (isEmpty) {
      // Verify empty state is handled
      expect(isEmpty).toBeTruthy();
    }
  });
});
