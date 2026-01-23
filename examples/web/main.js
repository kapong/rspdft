// Import from the wasm package
import init, { PdfTemplate, ThaiWordcut, ThaiFormatter } from './pkg/rspdft_wasm.js';

// Global state
let wasmReady = false;
let wordcut = null;
let template = null;
let loadedFonts = [];

// DOM Elements
const elements = {
    templateJson: document.getElementById('templateJson'),
    basePdf: document.getElementById('basePdf'),
    fontFile: document.getElementById('fontFile'),
    fontName: document.getElementById('fontName'),
    addFont: document.getElementById('addFont'),
    fontsList: document.getElementById('fontsList'),
    dataJson: document.getElementById('dataJson'),
    loadSampleTemplate: document.getElementById('loadSampleTemplate'),
    loadSampleData: document.getElementById('loadSampleData'),
    renderPdf: document.getElementById('renderPdf'),
    renderBatch: document.getElementById('renderBatch'),
    status: document.getElementById('status'),
    error: document.getElementById('error'),
    downloads: document.getElementById('downloads')
};

// Initialize WASM
async function initWasm() {
    try {
        await init();
        wordcut = ThaiWordcut.embedded();
        wasmReady = true;
        updateStatus('WASM initialized with embedded Thai dictionary', 'success');
        checkReadyState();
    } catch (error) {
        showError('Failed to initialize WASM: ' + error.message);
        console.error('WASM init error:', error);
    }
}

// Update status message
function updateStatus(message, type = 'info') {
    const statusItem = document.createElement('div');
    statusItem.className = `status-item ${type}`;
    statusItem.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    elements.status.appendChild(statusItem);
    elements.status.scrollTop = elements.status.scrollHeight;
}

// Show error message
function showError(message) {
    elements.error.textContent = message;
    elements.error.classList.remove('hidden');
    updateStatus(message, 'error');
}

// Clear error message
function clearError() {
    elements.error.textContent = '';
    elements.error.classList.add('hidden');
}

// Check if all required resources are loaded
function checkReadyState() {
    const hasTemplate = template !== null;
    const hasFonts = loadedFonts.length > 0;
    const hasData = elements.dataJson.value.trim() !== '';

    elements.renderPdf.disabled = !(wasmReady && hasTemplate && hasFonts && hasData);
    elements.renderBatch.disabled = !(wasmReady && hasTemplate && hasFonts && hasData);

    if (hasTemplate && hasFonts && hasData) {
        updateStatus('Ready to render PDF', 'success');
    }
}

// Read file as ArrayBuffer
function readFileAsArrayBuffer(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = () => reject(new Error('Failed to read file'));
        reader.readAsArrayBuffer(file);
    });
}

// Read file as text
function readFileAsText(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = () => reject(new Error('Failed to read file'));
        reader.readAsText(file);
    });
}

// Load template JSON
async function loadTemplateJson(jsonText) {
    try {
        clearError();
        template = PdfTemplate.from_json(jsonText);
        updateStatus('Template loaded successfully', 'success');
        checkReadyState();
    } catch (error) {
        showError('Failed to load template: ' + error.message);
        console.error('Template load error:', error);
    }
}

// Load base PDF
async function loadBasePdf(bytes) {
    try {
        clearError();
        if (!template) {
            showError('Please load template JSON first');
            return;
        }
        template.loadBasePdf(new Uint8Array(bytes));
        template.setWordcut(wordcut);
        updateStatus('Base PDF loaded successfully', 'success');
        checkReadyState();
    } catch (error) {
        showError('Failed to load PDF: ' + error.message);
        console.error('PDF load error:', error);
    }
}

// Load font
async function loadFont(name, bytes) {
    try {
        clearError();
        if (!template) {
            showError('Please load template JSON first');
            return;
        }
        template.loadFont(name, new Uint8Array(bytes));
        loadedFonts.push({ name, size: bytes.length });
        updateFontsList();
        updateStatus(`Font "${name}" loaded (${formatBytes(bytes.length)})`, 'success');
        checkReadyState();
    } catch (error) {
        showError('Failed to load font: ' + error.message);
        console.error('Font load error:', error);
    }
}

// Update fonts list UI
function updateFontsList() {
    if (loadedFonts.length === 0) {
        elements.fontsList.innerHTML = '<li class="empty">No fonts loaded</li>';
        return;
    }

    elements.fontsList.innerHTML = loadedFonts.map(font => `
        <li>
            <span class="font-name">${font.name}</span>
            <span class="font-size">${formatBytes(font.size)}</span>
        </li>
    `).join('');
}

// Format bytes to human readable
function formatBytes(bytes) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

// Render PDF
function renderPdf(data) {
    try {
        clearError();
        const pdfBytes = template.render(data);
        return pdfBytes;
    } catch (error) {
        showError('Failed to render PDF: ' + error.message);
        console.error('Render error:', error);
        throw error;
    }
}

// Download PDF
function downloadPdf(bytes, filename) {
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

// Add download link to UI
function addDownloadLink(bytes, filename) {
    const blob = new Blob([bytes], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);

    // Remove empty message if present
    const emptyMsg = elements.downloads.querySelector('.empty');
    if (emptyMsg) {
        emptyMsg.remove();
    }

    const link = document.createElement('a');
    link.className = 'download-link';
    link.href = url;
    link.download = filename;
    link.innerHTML = `
        <span class="icon">📄</span>
        <span class="filename">${filename}</span>
        <span class="size">${formatBytes(bytes.length)}</span>
    `;
    elements.downloads.appendChild(link);
}

// Load sample template
async function loadSampleTemplate() {
    try {
        const response = await fetch('sample/template.json');
        const json = await response.text();
        elements.dataJson.value = json;
        await loadTemplateJson(json);
        updateStatus('Sample template loaded', 'success');
    } catch (error) {
        showError('Failed to load sample template: ' + error.message);
    }
}

// Load sample data
async function loadSampleData() {
    try {
        const response = await fetch('sample/data.json');
        const json = await response.text();
        elements.dataJson.value = json;
        updateStatus('Sample data loaded', 'success');
        checkReadyState();
    } catch (error) {
        showError('Failed to load sample data: ' + error.message);
    }
}

// Event Listeners
elements.templateJson.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    try {
        const json = await readFileAsText(file);
        await loadTemplateJson(json);
    } catch (error) {
        showError('Failed to read template file: ' + error.message);
    }
});

elements.basePdf.addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    try {
        const bytes = await readFileAsArrayBuffer(file);
        await loadBasePdf(bytes);
    } catch (error) {
        showError('Failed to read PDF file: ' + error.message);
    }
});

elements.addFont.addEventListener('click', async () => {
    const file = elements.fontFile.files[0];
    const name = elements.fontName.value.trim();

    if (!file) {
        showError('Please select a font file');
        return;
    }

    if (!name) {
        showError('Please enter a font name');
        return;
    }

    try {
        const bytes = await readFileAsArrayBuffer(file);
        await loadFont(name, bytes);
        // Clear inputs
        elements.fontFile.value = '';
        elements.fontName.value = '';
    } catch (error) {
        showError('Failed to read font file: ' + error.message);
    }
});

elements.loadSampleTemplate.addEventListener('click', loadSampleTemplate);
elements.loadSampleData.addEventListener('click', loadSampleData);

elements.dataJson.addEventListener('input', checkReadyState);

elements.renderPdf.addEventListener('click', async () => {
    try {
        elements.renderPdf.disabled = true;
        elements.renderPdf.textContent = 'Rendering...';

        const data = JSON.parse(elements.dataJson.value);
        const pdfBytes = renderPdf(data);

        const timestamp = new Date().toISOString().slice(0, 19).replace(/[:-]/g, '');
        const filename = `output_${timestamp}.pdf`;

        downloadPdf(pdfBytes, filename);
        addDownloadLink(pdfBytes, filename);
        updateStatus('PDF rendered successfully', 'success');
    } catch (error) {
        // Error already handled in renderPdf
    } finally {
        elements.renderPdf.disabled = false;
        elements.renderPdf.textContent = 'Render PDF';
        checkReadyState();
    }
});

elements.renderBatch.addEventListener('click', async () => {
    try {
        elements.renderBatch.disabled = true;
        elements.renderBatch.textContent = 'Rendering...';

        const baseData = JSON.parse(elements.dataJson.value);

        // Generate 3 variations
        const variations = [
            { ...baseData, _variation: 'original' },
            { ...baseData, customer: { ...baseData.customer, name: 'บริษัท ตัวอย่าง จำกัด' }, _variation: 'variation1' },
            { ...baseData, amount: baseData.amount * 2, _variation: 'variation2' }
        ];

        for (let i = 0; i < variations.length; i++) {
            const pdfBytes = renderPdf(variations[i]);
            const timestamp = new Date().toISOString().slice(0, 19).replace(/[:-]/g, '');
            const filename = `batch_${i + 1}_${timestamp}.pdf`;

            downloadPdf(pdfBytes, filename);
            addDownloadLink(pdfBytes, filename);
            updateStatus(`Batch ${i + 1}/3 rendered`, 'success');
        }

        updateStatus('Batch rendering complete (3 PDFs)', 'success');
    } catch (error) {
        // Error already handled in renderPdf
    } finally {
        elements.renderBatch.disabled = false;
        elements.renderBatch.textContent = 'Render Batch (3 variations)';
        checkReadyState();
    }
});

// Initialize on load
initWasm();