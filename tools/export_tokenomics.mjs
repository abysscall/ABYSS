#!/usr/bin/env node
/**
 * Regenerate data/tokenomics.json from abyss-node.
 * Run from repo root: node tools/export_tokenomics.mjs
 */
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const outPath = join(root, 'data', 'tokenomics.json');

const result = spawnSync('cargo', ['run', '-q', '-p', 'abyss-node', '--', 'tokenomics', '--json'], {
  cwd: root,
  encoding: 'utf8',
});

if (result.status !== 0) {
  console.error(result.stderr || result.stdout || 'cargo run failed');
  process.exit(result.status ?? 1);
}

function parseAc(value) {
  if (typeof value === 'number') return value;
  const cleaned = String(value || '')
    .replace(/AC/i, '')
    .replace(/,/g, '')
    .trim();
  const amount = Number(cleaned);
  if (!Number.isFinite(amount)) {
    throw new Error(`Invalid AC amount: ${value}`);
  }
  return amount;
}

const tokenomics = JSON.parse(result.stdout);
tokenomics.max_supply_ac = parseAc(tokenomics.max_supply_ac);
tokenomics.team_reserve_ac = parseAc(tokenomics.team_reserve_ac);
tokenomics.public_sale_ac = parseAc(tokenomics.public_sale_ac);
for (const allocation of tokenomics.allocations || []) {
  allocation.amount_ac = parseAc(allocation.amount_ac);
}
for (const round of tokenomics.sale_rounds || []) {
  round.token_cap_ac = parseAc(round.token_cap_ac);
}
tokenomics._source = 'abyss-node tokenomics --json';

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(outPath, JSON.stringify(tokenomics, null, 2) + '\n', 'utf8');
console.log('Wrote', outPath);
