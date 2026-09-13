import { test, expect } from '@playwright/test';

test('typing a value and pressing Enter commits it and advances the active cell', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.type('5');
  await page.keyboard.press('Enter');

  await expect(page.locator('#cell-A1')).toHaveText('5');
  await expect(page.locator('#cell-A2')).toHaveClass(/active/);
});

test('F2 re-opens a cell for editing with its previous input', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.type('5');
  await page.keyboard.press('Enter');
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('F2');
  await page.keyboard.press('Backspace');
  await page.keyboard.type('9');
  await page.keyboard.press('Enter');

  await expect(page.locator('#cell-A1')).toHaveText('9');
});

test('mode indicator reflects READY, EDIT, and back to READY', async ({ page }) => {
  await page.goto('/');
  await expect(page.locator('#mode-indicator')).toHaveText('READY');
  await page.keyboard.type('5');
  await expect(page.locator('#mode-indicator')).toHaveText('EDIT');
  await page.keyboard.press('Enter');
  await expect(page.locator('#mode-indicator')).toHaveText('READY');
});

test('POINT-mode arrow navigation builds a @SUM formula referencing two cells', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.type('3');
  await page.keyboard.press('Enter'); // A1 = 3
  await page.keyboard.type('4');
  await page.keyboard.press('Enter'); // A2 = 4

  await page.keyboard.press('F5');
  await page.keyboard.type('B1');
  await page.keyboard.press('Enter'); // active -> B1

  await page.keyboard.type('@SUM(');
  await page.keyboard.press('ArrowLeft'); // POINT at A1
  await expect(page.locator('#mode-indicator')).toHaveText('POINT');
  await page.keyboard.type(',');
  await page.keyboard.press('ArrowLeft'); // POINT at A1 again
  await page.keyboard.press('ArrowDown'); // move to A2
  await page.keyboard.type(')');
  await page.keyboard.press('Enter');

  await expect(page.locator('#cell-B1')).toHaveText('7');
});

test('a circular reference displays ERR and the mode indicator reads ERROR', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.type('=B1');
  await page.keyboard.press('Enter'); // A1 = @formula referencing B1, active -> A2
  await page.keyboard.press('F5');
  await page.keyboard.type('B1');
  await page.keyboard.press('Enter'); // active -> B1
  await page.keyboard.type('=A1');
  await page.keyboard.press('Enter'); // B1 = @formula referencing A1 -> cycle

  await expect(page.locator('#cell-A1')).toHaveText('ERR');
  await page.keyboard.press('F5');
  await page.keyboard.type('A1');
  await page.keyboard.press('Enter'); // jump back to A1 to read its mode
  await expect(page.locator('#mode-indicator')).toHaveText('ERROR');
});

test('/Range Name defines a named range usable in a later formula', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.type('3');
  await page.keyboard.press('Enter'); // A1 = 3
  await page.keyboard.type('4');
  await page.keyboard.press('Enter'); // A2 = 4
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('ArrowUp'); // active -> A1

  await page.keyboard.press('/');
  await page.keyboard.press('r');
  await page.keyboard.press('ArrowDown'); // extend range to A2
  await page.keyboard.press('Enter');
  await page.keyboard.type('SALES');
  await page.keyboard.press('Enter');

  await page.keyboard.press('F5');
  await page.keyboard.type('B1');
  await page.keyboard.press('Enter');
  await page.keyboard.type('@SUM(SALES)');
  await page.keyboard.press('Enter');

  await expect(page.locator('#cell-B1')).toHaveText('7');
});

test('F5 GoTo jumps the active cell to a typed reference', async ({ page }) => {
  await page.goto('/');
  await page.keyboard.press('F5');
  await page.keyboard.type('C10');
  await page.keyboard.press('Enter');

  await expect(page.locator('#cell-C10')).toHaveClass(/active/);
});
