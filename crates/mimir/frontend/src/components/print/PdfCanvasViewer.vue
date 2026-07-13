<template>
  <div ref="containerRef" class="pdf-canvas-viewer">
    <div
      v-for="page in pages"
      :key="page.num"
      class="pdf-page"
      :data-page="page.num"
      :style="{ aspectRatio: `${page.width} / ${page.height}` }"
    >
      <canvas :ref="(el) => setCanvasRef(page.num, el as HTMLCanvasElement | null)"></canvas>
    </div>
  </div>
</template>

<script setup lang="ts">
// Canvas-based PDF preview. The Tauri webview on macOS (WKWebView) cannot
// render PDFs inside iframes/subframes, and the app CSP blocks blob: frames,
// so blob-URL iframes show a blank pane. pdf.js draws to canvases instead,
// which works in every webview. Pages render lazily as they scroll into view.
import { ref, watch, onBeforeUnmount, nextTick } from 'vue'
import type { PDFDocumentProxy, PDFDocumentLoadingTask } from 'pdfjs-dist'

// pdf.js is loaded lazily: its canvas module breaks under jsdom, so a static
// import would take down every unit test that mounts a component importing
// this one. The dynamic import also keeps it out of the main bundle.
async function loadPdfjs() {
  const [pdfjs, worker] = await Promise.all([
    import('pdfjs-dist'),
    import('pdfjs-dist/build/pdf.worker.mjs?url'),
  ])
  pdfjs.GlobalWorkerOptions.workerSrc = worker.default
  return pdfjs
}

const props = defineProps<{
  /** Base64-encoded PDF data */
  pdfBase64: string
}>()

interface PageInfo {
  num: number
  width: number
  height: number
}

const containerRef = ref<HTMLElement | null>(null)
const pages = ref<PageInfo[]>([])
const canvasRefs = new Map<number, HTMLCanvasElement>()
const renderedPages = new Set<number>()

let doc: PDFDocumentProxy | null = null
let loadingTask: PDFDocumentLoadingTask | null = null
let observer: IntersectionObserver | null = null
let loadToken = 0

function setCanvasRef(num: number, el: HTMLCanvasElement | null) {
  if (el) canvasRefs.set(num, el)
  else canvasRefs.delete(num)
}

async function renderPage(num: number) {
  if (!doc || renderedPages.has(num)) return
  renderedPages.add(num)
  const canvas = canvasRefs.get(num)
  if (!canvas) return
  const page = await doc.getPage(num)
  const scale = 1.5
  const viewport = page.getViewport({ scale })
  const dpr = window.devicePixelRatio || 1
  canvas.width = viewport.width * dpr
  canvas.height = viewport.height * dpr
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.scale(dpr, dpr)
  await page.render({ canvas, canvasContext: ctx, viewport }).promise
}

async function load() {
  const token = ++loadToken
  cleanup()
  if (!props.pdfBase64) return

  const pdfjs = await loadPdfjs()
  const bytes = Uint8Array.from(atob(props.pdfBase64), (c) => c.charCodeAt(0))
  const task = pdfjs.getDocument({ data: bytes })
  const loaded = await task.promise
  if (token !== loadToken) {
    task.destroy()
    return
  }
  loadingTask = task
  doc = loaded

  const first = await doc.getPage(1)
  const { width, height } = first.getViewport({ scale: 1 })
  pages.value = Array.from({ length: doc.numPages }, (_, i) => ({
    num: i + 1,
    width,
    height,
  }))

  await nextTick()
  observer = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          const num = Number((entry.target as HTMLElement).dataset.page)
          renderPage(num)
          observer?.unobserve(entry.target)
        }
      }
    },
    { root: containerRef.value, rootMargin: '150%' }
  )
  containerRef.value
    ?.querySelectorAll('.pdf-page')
    .forEach((el) => observer?.observe(el))
}

function cleanup() {
  observer?.disconnect()
  observer = null
  renderedPages.clear()
  pages.value = []
  loadingTask?.destroy()
  loadingTask = null
  doc = null
}

watch(() => props.pdfBase64, load, { immediate: true })
onBeforeUnmount(cleanup)
</script>

<style scoped>
.pdf-canvas-viewer {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md, 12px);
  padding: var(--spacing-md, 12px);
  background: var(--color-surface-variant, #e5e5e5);
}

.pdf-page {
  width: min(100%, 900px);
  background: white;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.25);
  flex-shrink: 0;
}

.pdf-page canvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>
