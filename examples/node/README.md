# rspdft Node.js Example

Batch PDF generation using rspdft WASM in Node.js.

## Prerequisites

1. Build WASM for Node.js:
   ```bash
   cd crates/wasm
   wasm-pack build --target nodejs
   ```

2. Link the package:
   ```bash
   cd examples/node
   ln -s ../../crates/wasm/pkg pkg
   ```

## Usage

### Basic Usage

```bash
node render.mjs \
  --template sample/template.json \
  --pdf path/to/base.pdf \
  --font sarabun:../../fonts/THSarabunNew.ttf \
  --data sample/records.json \
  --output ./output
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--template` | `-t` | Template JSON file (required) |
| `--pdf` | `-p` | Base PDF file (required) |
| `--font` | `-f` | Font in `name:path` format (can repeat) |
| `--data` | `-d` | JSON file with array of records (required) |
| `--output` | `-o` | Output directory (default: `./output`) |
| `--help` | `-h` | Show help |

### Data Format

The data file should contain a JSON array. Each object can include:
- `_filename`: Custom output filename (optional, defaults to `output_001.pdf`, etc.)
- Other fields matching template bindings (e.g., `$.customer.name`)

## Example Output

```
Initializing WASM...
Loading Thai word segmentation...
Loading template...
Loading base PDF...
Loading font "sarabun" from ../../fonts/THSarabunNew.ttf...
Loading data records...

Rendering 3 PDFs...
  [1/3] invoice_001.pdf ✓
  [2/3] invoice_002.pdf ✓
  [3/3] invoice_003.pdf ✓

Done!
```

## Features

- **Load once, render many**: Resources loaded once, then reused for all records
- **Custom filenames**: Use `_filename` field in data records
- **Thai text support**: Embedded dictionary for word segmentation
- **Thai formatting**: Baht text, Thai dates
- **Progress tracking**: See progress for each record