import { test, expect } from '@playwright/test';

test('typing into A1 and A2 updates the B1 sum via the WASM core', async ({ page }) => {
  await page.goto('/');

  await page.fill('#a1', '3');
  await page.fill('#a2', '4');

  await expect(page.locator('#b1')).toHaveText('7');
});
