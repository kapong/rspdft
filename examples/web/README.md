# rspdft Browser Demo

A browser-based demonstration of the rspdft PDF template filling library with Thai language support.

## Setup

### 1. Build the WASM Package

First, build the WebAssembly package from the `crates/wasm` directory:

```bash
cd crates/wasm
wasm-pack build --target web
```

This will create a `pkg/` directory containing:
- `rspdft_wasm.js` - JavaScript bindings
- `rspdft_wasm_bg.wasm` - WebAssembly binary
- TypeScript definitions and package metadata

### 2. Link the Package

Make the WASM package available to the web demo. You have two options:

**Option A: Symlink (Recommended for Development)**

```bash
cd examples/web
ln -s ../../crates/wasm/pkg pkg
```

**Option B: Copy**

```bash
cd examples/web
cp -r ../../crates/wasm/pkg pkg
```

### 3. Serve the Demo

Use any static file server to serve the demo:

**Using Python:**
```bash
python3 -m http.server 8080
```

**Using Node.js (http-server):**
```bash
npx http-server -p 8080
```

**Using PHP:**
```bash
php -S localhost:8080
```

### 4. Open in Browser

Navigate to:
```
http://localhost:8080/examples/web/
```

## Usage

### Step 1: Load Resources

1. **Template JSON**: Click "Load Sample Template" to use the included sample, or upload your own template JSON file.

2. **Base PDF**: Upload a blank PDF template that matches your template definition. The sample template expects an A4-sized blank PDF.

3. **Font File**: 
   - Select a TTF or OTF font file (e.g., THSarabunNew.ttf for Thai text)
   - Enter a font name (e.g., "sarabun")
   - Click "Add Font"
   - You can add multiple fonts if needed

### Step 2: Input Data

- Click "Load Sample Data" to use the included sample data, or
- Enter your own JSON data in the textarea

### Step 3: Render PDF

- **Render PDF**: Generates a single PDF with the current data
- **Render Batch**: Generates 3 PDF variations with modified data

### Step 4: Download

Generated PDFs will appear in the Downloads section. Click any link to download, or they will auto-download when rendered.

## Sample Template

The included sample template demonstrates:

- **Text blocks**: Static text ("Invoice") and bound data (customer name)
- **Number formatting**: Thai Baht amount with comma separators
- **Thai Baht text**: Automatic conversion to Thai text (e.g., "หนึ่งร้อยบาทถ้วน")
- **Thai date formatting**: Long format Thai dates (e.g., "23 มกราคม 2568")
- **QR codes**: Embedded QR code generation

## Features Demonstrated

- ✅ Embedded Thai dictionary (no external file needed)
- ✅ Template JSON parsing
- ✅ Base PDF loading
- ✅ Font loading (TTF/OTF)
- ✅ Data binding with JSONPath (`$.customer.name`)
- ✅ Number formatting
- ✅ Thai Baht text conversion
- ✅ Thai date formatting
- ✅ QR code generation
- ✅ Batch rendering

## Troubleshooting

### WASM Not Loading

- Ensure the `pkg/` directory exists and contains `rspdft_wasm.js` and `rspdft_wasm_bg.wasm`
- Check browser console for CORS errors (use a local server, don't open `file://` directly)
- Verify the symlink or copy was created correctly

### Font Not Working

- Ensure the font name matches the `font.family` in your template JSON
- Use TTF or OTF format fonts
- For Thai text, use a font that supports Thai characters (e.g., THSarabun, Sarabun, Noto Sans Thai)

### PDF Rendering Errors

- Check the browser console for detailed error messages
- Verify your template JSON is valid
- Ensure all required fonts are loaded
- Make sure the base PDF exists and is a valid PDF file

## Browser Compatibility

Works in all modern browsers that support:
- WebAssembly
- ES6 modules
- FileReader API
- Blob API

Tested on:
- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## Development

To modify the demo:

1. Edit `index.html` for structure
2. Edit `main.js` for JavaScript logic
3. Edit `style.css` for styling
4. Update `sample/template.json` and `sample/data.json` for new samples

After modifying the WASM code in `crates/wasm/`, rebuild with:

```bash
cd crates/wasm && wasm-pack build --target web
```

The symlink will automatically pick up the new build.