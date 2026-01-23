#!/usr/bin/env node
/**
 * rspdft Node.js CLI Example
 *
 * Demonstrates batch PDF generation:
 * 1. Load template, PDF, and fonts once
 * 2. Render multiple records from a JSON array
 * 3. Save each PDF to output directory
 *
 * Usage:
 *   node render.mjs --template template.json --pdf base.pdf --font sarabun:font.ttf --data records.json --output ./output
 */

import { readFile, writeFile, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import { dirname, join, basename } from 'path';
import { parseArgs } from 'util';

// Import WASM (assumes pkg is symlinked from crates/wasm/pkg)
import init, { PdfTemplate, ThaiWordcut } from './pkg/rspdft_wasm.js';

// Parse CLI arguments
const { values: args } = parseArgs({
  options: {
    template: { type: 'string', short: 't' },
    pdf: { type: 'string', short: 'p' },
    font: { type: 'string', short: 'f', multiple: true },  // name:path pairs
    data: { type: 'string', short: 'd' },
    output: { type: 'string', short: 'o', default: './output' },
    help: { type: 'boolean', short: 'h' }
  }
});

// Show help
function showHelp() {
  console.log(`
rspdft Node.js CLI Example

Usage:
  node render.mjs [options]

Options:
  -t, --template <path>   Template JSON file (required)
  -p, --pdf <path>        Base PDF file (required)
  -f, --font <name:path>  Font file (can be specified multiple times)
  -d, --data <path>       Data JSON file with array of records (required)
  -o, --output <dir>      Output directory (default: ./output)
  -h, --help              Show this help

Example:
  node render.mjs -t template.json -p base.pdf -f sarabun:fonts/THSarabunNew.ttf -d records.json -o ./output
  `);
}

async function main() {
  if (args.help) {
    showHelp();
    process.exit(0);
  }

  // Validate required args
  if (!args.template || !args.pdf || !args.data) {
    console.error('Error: --template, --pdf, and --data are required');
    showHelp();
    process.exit(1);
  }

  console.log('Initializing WASM...');
  await init();

  console.log('Loading Thai word segmentation...');
  const wordcut = ThaiWordcut.embedded();

  console.log('Loading template...');
  const templateJson = await readFile(args.template, 'utf-8');
  const template = PdfTemplate.fromJson(templateJson);

  console.log('Loading base PDF...');
  const pdfBytes = await readFile(args.pdf);
  template.loadBasePdf(new Uint8Array(pdfBytes));
  template.setWordcut(wordcut);

  // Load fonts
  if (args.font) {
    for (const fontArg of args.font) {
      const [name, path] = fontArg.split(':');
      if (!name || !path) {
        console.error(`Invalid font format: ${fontArg}. Use name:path`);
        process.exit(1);
      }
      console.log(`Loading font "${name}" from ${path}...`);
      const fontBytes = await readFile(path);
      template.loadFont(name, new Uint8Array(fontBytes));
    }
  }

  console.log('Loading data records...');
  const dataJson = await readFile(args.data, 'utf-8');
  const records = JSON.parse(dataJson);

  if (!Array.isArray(records)) {
    console.error('Error: Data file must contain a JSON array of records');
    process.exit(1);
  }

  // Create output directory
  if (!existsSync(args.output)) {
    await mkdir(args.output, { recursive: true });
  }

  console.log(`\nRendering ${records.length} PDFs...`);

  for (let i = 0; i < records.length; i++) {
    const record = records[i];
    const filename = record._filename || `output_${String(i + 1).padStart(3, '0')}.pdf`;
    const outputPath = join(args.output, filename);

    try {
      const pdfBytes = template.render(record);
      await writeFile(outputPath, Buffer.from(pdfBytes));
      console.log(`  [${i + 1}/${records.length}] ${filename} ✓`);
    } catch (err) {
      console.error(`  [${i + 1}/${records.length}] ${filename} ✗ - ${err.message}`);
    }
  }

  console.log('\nDone!');
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});