// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

import init, { Spreadsheet } from './wasm/web123_core.js';

async function main() {
  await init();
  const sheet = new Spreadsheet();

  const a1 = document.querySelector<HTMLInputElement>('#a1')!;
  const a2 = document.querySelector<HTMLInputElement>('#a2')!;
  const b1 = document.querySelector<HTMLSpanElement>('#b1')!;

  function recalc() {
    sheet.setCell('A1', a1.value);
    sheet.setCell('A2', a2.value);
    sheet.setCell('B1', '@SUM(A1,A2)');
    b1.textContent = sheet.getDisplay('B1');
  }

  a1.addEventListener('input', recalc);
  a2.addEventListener('input', recalc);
  recalc();
}

main();
