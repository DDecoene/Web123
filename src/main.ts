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
  const mode = sheet.getMode();
  const isEditingLike = mode === 'EDIT' || mode === 'POINT';
  const pointCell = mode === 'POINT' ? sheet.getPointCell() : null;
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const addr = `${colLabel(c)}${r + 1}`;
      const cell = document.getElementById(`cell-${addr}`)!;
      cell.textContent = sheet.getDisplay(addr);
      cell.classList.toggle('active', addr === activeCell);
      cell.classList.toggle('pointing', addr === pointCell);
    }
  }
  document.getElementById('mode-indicator')!.textContent = mode;
  document.getElementById('edit-line')!.textContent = isEditingLike ? sheet.getEditBuffer() : '';
}

// The WASM module's fetch/compile/instantiate is asynchronous, so there is an
// unavoidable gap between the page becoming visible/interactive and the sheet
// being ready to handle keys. Attach the listener immediately and queue any
// keys that arrive during that gap so they aren't silently lost.
let sheet: Spreadsheet | null = null;
const pendingKeys: string[] = [];

function handleForwardedKey(key: string) {
  if (sheet) {
    sheet.handleKey(key);
    renderGrid(sheet);
  } else {
    pendingKeys.push(key);
  }
}

window.addEventListener('keydown', (e) => {
  // Let the browser handle its own Ctrl/Cmd/Alt shortcuts (copy, reload,
  // new tab, etc.) instead of swallowing them and forwarding the bare key
  // into the spreadsheet core.
  if (e.ctrlKey || e.metaKey || e.altKey) {
    return;
  }
  if (e.key.length === 1 || FORWARDED_KEYS.has(e.key)) {
    e.preventDefault();
    handleForwardedKey(e.key);
  }
});

async function main() {
  await init();
  const newSheet = new Spreadsheet();
  await newSheet.loadFromStorage();
  sheet = newSheet;

  const gridContainer = document.querySelector<HTMLDivElement>('#grid-container')!;
  buildGrid(gridContainer);

  for (const key of pendingKeys) {
    sheet.handleKey(key);
  }
  pendingKeys.length = 0;

  renderGrid(sheet);
}

main().catch((e) => {
  console.error(e);
});
