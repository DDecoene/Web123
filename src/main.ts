// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

import init, { Spreadsheet } from './wasm/web123_core.js';

const COLS = 26;
const ROWS = 100;

const FORWARDED_KEYS = new Set([
  'ArrowUp',
  'ArrowDown',
  'ArrowLeft',
  'ArrowRight',
  'Enter',
  'Escape',
  'Backspace',
  'F2',
  'F5',
  'F9',
]);

function colLabel(i: number): string {
  return String.fromCharCode('A'.charCodeAt(0) + i);
}

function buildGrid(container: HTMLElement) {
  const table = document.createElement('table');
  const headerRow = document.createElement('tr');
  headerRow.appendChild(document.createElement('th'));
  for (let c = 0; c < COLS; c++) {
    const th = document.createElement('th');
    th.textContent = colLabel(c);
    headerRow.appendChild(th);
  }
  table.appendChild(headerRow);

  for (let r = 0; r < ROWS; r++) {
    const row = document.createElement('tr');
    const rowHeader = document.createElement('th');
    rowHeader.textContent = String(r + 1);
    row.appendChild(rowHeader);
    for (let c = 0; c < COLS; c++) {
      const td = document.createElement('td');
      td.id = `cell-${colLabel(c)}${r + 1}`;
      row.appendChild(td);
    }
    table.appendChild(row);
  }
  container.appendChild(table);
}

function renderGrid(sheet: Spreadsheet) {
  const activeCell = sheet.getActiveCell();
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const addr = `${colLabel(c)}${r + 1}`;
      const cell = document.getElementById(`cell-${addr}`)!;
      cell.textContent = sheet.getDisplay(addr);
      cell.classList.toggle('active', addr === activeCell);
    }
  }
  document.getElementById('mode-indicator')!.textContent = sheet.getMode();
}

async function main() {
  await init();
  const sheet = new Spreadsheet();

  const gridContainer = document.querySelector<HTMLDivElement>('#grid-container')!;
  buildGrid(gridContainer);
  renderGrid(sheet);

  window.addEventListener('keydown', (e) => {
    if (e.key.length === 1 || FORWARDED_KEYS.has(e.key)) {
      e.preventDefault();
      sheet.handleKey(e.key);
      renderGrid(sheet);
    }
  });
}

main();
