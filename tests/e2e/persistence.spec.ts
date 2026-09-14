// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

import { test, expect } from '@playwright/test';

test('a cell edit survives a page reload via IndexedDB', async ({ page }) => {
  await page.goto('/');

  // Type "42" into A1 (already the active cell on boot) and commit it.
  await page.keyboard.type('42');
  await page.keyboard.press('Enter');
  await expect(page.locator('#cell-A1')).toHaveText('42');

  await page.reload();

  // loadFromStorage() runs before the first render, so the value should
  // already be there without any further interaction.
  await expect(page.locator('#cell-A1')).toHaveText('42');
});

test('a fresh browser context with no prior storage starts with an empty grid', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#cell-A1')).toHaveText('');
});
