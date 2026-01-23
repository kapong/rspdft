// BOJ45 Tax Form Demo - URL-based resource loading
import init, { PdfTemplate, ThaiWordcut } from './pkg/rspdft_wasm.js';

// State
let template = null;
let wordcut = null;
let sampleData = null;

// DOM Elements
const els = {
    formName: document.getElementById('formName'),
    resTemplate: document.getElementById('res-template'),
    resPdf: document.getElementById('res-pdf'),
    resFonts: document.getElementById('res-fonts'),
    dataJson: document.getElementById('dataJson'),
    renderPdf: document.getElementById('renderPdf'),
    resetData: document.getElementById('resetData'),
    error: document.getElementById('error'),
    downloads: document.getElementById('downloads')
};

// Update resource status UI
function updateResourceStatus(element, status, message) {
    element.className = `resource-item ${status}`;
    const icon = element.querySelector('.icon');
    const statusText = element.querySelector('.status');
    
    switch (status) {
        case 'loading':
            icon.textContent = '⏳';
            break;
        case 'success':
            icon.textContent = '✅';
            break;
        case 'error':
            icon.textContent = '❌';
            break;
    }
    statusText.textContent = message;
}

// Show error
function showError(message) {
    els.error.textContent = message;
    els.error.classList.remove('hidden');
    console.error(message);
}

// Clear error
function clearError() {
    els.error.textContent = '';
    els.error.classList.add('hidden');
}

// Format bytes
function formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return (bytes / Math.pow(k, i)).toFixed(1) + ' ' + sizes[i];
}

// Convert relative path to absolute (remove leading ./)
function toAbsolutePath(path) {
    return path.replace(/^\.\//, '/');
}

// Fetch with error handling
async function fetchResource(url, type = 'arrayBuffer') {
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.status}`);
    }
    return type === 'json' ? response.json() : 
           type === 'text' ? response.text() : 
           response.arrayBuffer();
}

// Initialize everything
async function initialize() {
    try {
        // 1. Initialize WASM
        await init();
        wordcut = ThaiWordcut.embedded();
        console.log('WASM initialized');

        // 2. Load config
        const config = await fetchResource('config.json', 'json');
        els.formName.textContent = config.name;
        console.log('Config loaded:', config);

        // 3. Load and parse template JSON
        updateResourceStatus(els.resTemplate, 'loading', 'Fetching...');
        const templateJson = await fetchResource(config.template, 'text');
        const templateData = JSON.parse(templateJson);
        template = PdfTemplate.fromJson(templateJson);
        updateResourceStatus(els.resTemplate, 'success', 'Loaded');
        console.log('Template loaded:', templateData);

        // 4. Load base PDF from path in template
        updateResourceStatus(els.resPdf, 'loading', 'Fetching...');
        const pdfPath = toAbsolutePath(templateData.template.source);
        const pdfBytes = await fetchResource(pdfPath);
        template.loadBasePdf(new Uint8Array(pdfBytes));
        template.setWordcut(wordcut);
        updateResourceStatus(els.resPdf, 'success', formatBytes(pdfBytes.byteLength));
        console.log('PDF loaded from:', pdfPath);

        // 5. Load fonts from paths in template
        const fonts = templateData.fonts || [];
        const fontStyles = ['regular', 'bold', 'italic', 'boldItalic'];
        let totalFonts = 0;
        let loadedFonts = 0;

        // Count total fonts
        for (const font of fonts) {
            for (const style of fontStyles) {
                if (font[style]) totalFonts++;
            }
        }

        updateResourceStatus(els.resFonts, 'loading', `0/${totalFonts}`);

        // Load each font
        for (const font of fonts) {
            for (const style of fontStyles) {
                if (font[style]) {
                    const fontPath = toAbsolutePath(font[style]);
                    const fontBytes = await fetchResource(fontPath);
                    
                    // Build font key: "sarabun", "sarabun-bold", "sarabun-italic", "sarabun-bold-italic"
                    const fontKey = style === 'regular' ? font.id : `${font.id}-${style.replace('Italic', '-italic').replace('bold', 'bold')}`;
                    const normalizedKey = style === 'regular' ? font.id :
                                          style === 'bold' ? `${font.id}-bold` :
                                          style === 'italic' ? `${font.id}-italic` :
                                          `${font.id}-bold-italic`;
                    
                    template.loadFont(normalizedKey, new Uint8Array(fontBytes));
                    loadedFonts++;
                    updateResourceStatus(els.resFonts, 'loading', `${loadedFonts}/${totalFonts}`);
                    console.log(`Font loaded: ${normalizedKey} from ${fontPath}`);
                }
            }
        }
        updateResourceStatus(els.resFonts, 'success', `${loadedFonts} fonts`);

        // 6. Load sample data
        sampleData = await fetchResource(config.sampleData, 'text');
        els.dataJson.value = sampleData;
        console.log('Sample data loaded');

        // 7. Ready!
        els.renderPdf.disabled = false;
        console.log('Ready to render');

    } catch (error) {
        showError('Initialization failed: ' + error.message);
        console.error('Init error:', error);
    }
}

// Render PDF
function handleRender() {
    try {
        clearError();
        els.renderPdf.disabled = true;
        els.renderPdf.querySelector('.btn-text').textContent = 'Rendering...';

        const data = JSON.parse(els.dataJson.value);
        const pdfBytes = template.render(data);

        // Create download
        const blob = new Blob([pdfBytes], { type: 'application/pdf' });
        const url = URL.createObjectURL(blob);
        const timestamp = new Date().toISOString().slice(0, 19).replace(/[:-]/g, '');
        const filename = `boj45_${timestamp}.pdf`;

        // Auto-download
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        a.click();

        // Add to downloads list
        const emptyMsg = els.downloads.querySelector('.empty');
        if (emptyMsg) emptyMsg.remove();

        const link = document.createElement('a');
        link.className = 'download-link';
        link.href = url;
        link.download = filename;
        link.innerHTML = `
            <span class="icon">📄</span>
            <span class="filename">${filename}</span>
            <span class="size">${formatBytes(pdfBytes.length)}</span>
        `;
        els.downloads.appendChild(link);

        console.log('PDF rendered:', filename);

    } catch (error) {
        showError('Render failed: ' + error.message);
        console.error('Render error:', error);
    } finally {
        els.renderPdf.disabled = false;
        els.renderPdf.querySelector('.btn-text').textContent = 'Render PDF';
    }
}

// Reset data to sample
function handleReset() {
    if (sampleData) {
        els.dataJson.value = sampleData;
        clearError();
    }
}

// Event listeners
els.renderPdf.addEventListener('click', handleRender);
els.resetData.addEventListener('click', handleReset);

// Start initialization
initialize();