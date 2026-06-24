import { expect, test } from '@playwright/test';

test('page loads and sets title', async ({ page }) => {
  await page.goto('/');
  // Since Tauri API is not available in a pure browser context,
  // we just ensure the page loads (even if it renders an error state or SILO).
  await expect(page).toHaveTitle(/SILO|Tauri/);
});
