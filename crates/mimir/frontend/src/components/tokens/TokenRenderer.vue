<template>
  <div class="token-layer" :style="layerStyle">
    <div
      v-for="token in displayTokens"
      :key="token.id"
      class="token"
      :class="{
        'token-hidden': !token.visible_to_players && showHidden,
        'token-selected': selectedTokenId === token.id,
        'token-dragging': draggingTokenId === token.id,
        'token-has-light': hasActiveLight(token.id),
        'token-dead': isDead(token.id)
      }"
      :style="getTokenStyle(token)"
      :title="token.name"
      @mousedown.stop="handleMouseDown($event, token)"
      @click.stop="handleClick($event, token)"
      @contextmenu.prevent="$emit('token-context', $event, token)"
    >
      <!-- Token image (if available) -->
      <img
        v-if="hasTokenImage(token.id)"
        :src="getTokenImage(token.id)"
        class="token-image"
        alt=""
      />
      <!-- Fallback label (if no image) -->
      <span v-else class="token-label">{{ getTokenLabel(token) }}</span>
      <!-- Light source indicator badge -->
      <span
        v-if="hasLight(token.id)"
        class="light-badge"
        :class="{ 'light-active': hasActiveLight(token.id) }"
        :title="hasActiveLight(token.id) ? 'Light source (active)' : 'Light source (inactive)'"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
          <path d="M10 1a1 1 0 011 1v1a1 1 0 11-2 0V2a1 1 0 011-1zM5.05 3.636a1 1 0 011.414 0l.707.707a1 1 0 11-1.414 1.414l-.707-.707a1 1 0 010-1.414zM16.95 3.636a1 1 0 010 1.414l-.707.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM10 6a4 4 0 100 8 4 4 0 000-8zM2 11a1 1 0 011-1h1a1 1 0 110 2H3a1 1 0 01-1-1zM16 11a1 1 0 011-1h1a1 1 0 110 2h-1a1 1 0 01-1-1zM5.05 18.364a1 1 0 010-1.414l.707-.707a1 1 0 111.414 1.414l-.707.707a1 1 0 01-1.414 0zM16.95 18.364a1 1 0 01-1.414 0l-.707-.707a1 1 0 111.414-1.414l.707.707a1 1 0 010 1.414zM10 15a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1z"/>
        </svg>
      </span>
      <!-- Dead indicator (skull) -->
      <span
        v-if="isDead(token.id)"
        class="dead-badge"
        title="Dead"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12c0 3.69 2.47 6.86 6 8.25V22h8v-1.75c3.53-1.39 6-4.56 6-8.25 0-5.52-4.48-10-10-10zM8.5 14c-.83 0-1.5-.67-1.5-1.5S7.67 11 8.5 11s1.5.67 1.5 1.5S9.33 14 8.5 14zm3.5 4h-2v-2h2v2zm2 0h-2v-2h2v2zm1-4c-.83 0-1.5-.67-1.5-1.5s.67-1.5 1.5-1.5 1.5.67 1.5 1.5-.67 1.5-1.5 1.5z"/>
        </svg>
      </span>
      <!-- Visibility badge -->
      <span
        v-if="!token.visible_to_players && showHidden"
        class="visibility-badge"
        title="Hidden from players"
      >
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M3.28 2.22a.75.75 0 00-1.06 1.06l14.5 14.5a.75.75 0 101.06-1.06l-1.745-1.745a10.029 10.029 0 003.3-4.38 1.651 1.651 0 000-1.185A10.004 10.004 0 009.999 3a9.956 9.956 0 00-4.744 1.194L3.28 2.22zM7.752 6.69l1.092 1.092a2.5 2.5 0 013.374 3.373l1.091 1.092a4 4 0 00-5.557-5.557z" clip-rule="evenodd" />
          <path d="M10.748 13.93l2.523 2.523a9.987 9.987 0 01-3.27.547c-4.258 0-7.894-2.66-9.337-6.41a1.651 1.651 0 010-1.186A10.007 10.007 0 012.839 6.02L6.07 9.252a4 4 0 004.678 4.678z" />
        </svg>
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Token, TokenSize, TokenType } from '@/types/api'
import { TOKEN_SIZE_GRID_SQUARES, TOKEN_TYPE_COLORS } from '@/types/api'

/** Token IDs that have an active light source */
interface TokenLightInfo {
  tokenId: string
  isActive: boolean
}

interface Props {
  tokens: Token[]
  gridSizePx: number
  baseScale?: number
  showHidden?: boolean
  selectedTokenId?: string | null
  draggingTokenId?: string | null
  dragOffset?: { x: number; y: number } | null
  interactive?: boolean
  /** Light source info for tokens */
  tokenLights?: TokenLightInfo[]
  /** Token IDs that are marked as dead */
  deadTokenIds?: string[]
  /** Map of token_id -> base64 image data URL */
  tokenImages?: Map<string, string>
}

const props = withDefaults(defineProps<Props>(), {
  baseScale: 1,
  showHidden: true,
  selectedTokenId: null,
  draggingTokenId: null,
  dragOffset: null,
  interactive: true,
  tokenLights: () => [],
  deadTokenIds: () => [],
  tokenImages: () => new Map()
})

// Get token image if available
function getTokenImage(tokenId: string): string | undefined {
  return props.tokenImages.get(tokenId)
}

// Check if token has an image
function hasTokenImage(tokenId: string): boolean {
  return props.tokenImages.has(tokenId)
}

// Check if a token has a light source and if it's active
function getTokenLight(tokenId: string): TokenLightInfo | undefined {
  return props.tokenLights.find(l => l.tokenId === tokenId)
}

function hasActiveLight(tokenId: string): boolean {
  const light = getTokenLight(tokenId)
  return light?.isActive ?? false
}

function hasLight(tokenId: string): boolean {
  return props.tokenLights.some(l => l.tokenId === tokenId)
}

function isDead(tokenId: string): boolean {
  return props.deadTokenIds.includes(tokenId)
}

const emit = defineEmits<{
  'token-click': [token: Token]
  'token-context': [event: MouseEvent, token: Token]
  'token-drag-start': [event: MouseEvent, token: Token]
}>()

// Track if we've moved enough to consider it a drag
const hasMoved = ref(false)
const mouseDownPos = ref<{ x: number; y: number } | null>(null)

function handleMouseDown(event: MouseEvent, token: Token) {
  if (event.button !== 0 || !props.interactive) return

  hasMoved.value = false
  mouseDownPos.value = { x: event.clientX, y: event.clientY }

  // Emit drag start - parent will handle actual dragging
  emit('token-drag-start', event, token)
}

function handleClick(event: MouseEvent, token: Token) {
  // Only emit click if we haven't dragged
  if (!hasMoved.value) {
    emit('token-click', token)
  }
  hasMoved.value = false
  mouseDownPos.value = null
}

// Expose hasMoved so parent can set it
defineExpose({ setHasMoved: (val: boolean) => { hasMoved.value = val } })

// Filter tokens - if showHidden is false, only show visible tokens
const displayTokens = computed(() => {
  if (props.showHidden) {
    return props.tokens
  }
  return props.tokens.filter(t => t.visible_to_players)
})

// Layer style (sized to match the map)
const layerStyle = computed(() => ({
  position: 'absolute' as const,
  top: 0,
  left: 0,
  width: '100%',
  height: '100%',
  pointerEvents: props.interactive ? 'auto' as const : 'none' as const
}))

// Scale tokens to 85% of grid size so they fit within grid cells
const TOKEN_SCALE = 0.85

// Get token display style
function getTokenStyle(token: Token) {
  const gridSquares = TOKEN_SIZE_GRID_SQUARES[token.size as TokenSize] || 1
  const tokenSizePx = gridSquares * props.gridSizePx * props.baseScale * TOKEN_SCALE
  const color = token.color || TOKEN_TYPE_COLORS[token.token_type as TokenType] || '#666666'

  // Position at token center, offset by half the token size
  let left = (token.x * props.baseScale) - (tokenSizePx / 2)
  let top = (token.y * props.baseScale) - (tokenSizePx / 2)

  // Apply drag offset if this token is being dragged
  if (props.draggingTokenId === token.id && props.dragOffset) {
    left += props.dragOffset.x
    top += props.dragOffset.y
  }

  return {
    left: left + 'px',
    top: top + 'px',
    width: tokenSizePx + 'px',
    height: tokenSizePx + 'px',
    backgroundColor: color,
    borderColor: color,
    fontSize: Math.max(tokenSizePx * 0.24, 16) + 'px'
  }
}

// Get token label - full name
function getTokenLabel(token: Token): string {
  return token.name
}
</script>

<style scoped>
.token-layer {
  pointer-events: none;
}

.token {
  position: absolute;
  border-radius: 50%;
  border: 3px solid;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
  transition: transform 0.1s, box-shadow 0.1s;
  pointer-events: auto;
  cursor: pointer;
}

.token:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.5);
  z-index: 10;
}

.token-hidden {
  opacity: 0.5;
}

.token-selected {
  box-shadow: 0 0 0 3px white, 0 0 0 6px var(--color-primary-500, #3b82f6);
  z-index: 20;
}

.token-dragging {
  opacity: 0.8;
  transform: scale(1.1);
  z-index: 100;
  cursor: grabbing;
  transition: none;
}

.token-image {
  width: 100%;
  height: 100%;
  border-radius: 50%;
  object-fit: cover;
  pointer-events: none;
}

.token-label {
  font-weight: 600;
  font-size: 0.65em;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.7);
  user-select: none;
  text-align: center;
  line-height: 1.1;
  max-width: 90%;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-word;
}

.visibility-badge {
  position: absolute;
  top: -4px;
  right: -4px;
  width: 16px;
  height: 16px;
  background: rgba(0, 0, 0, 0.7);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.visibility-badge svg {
  width: 10px;
  height: 10px;
  fill: #fbbf24;
}

/* Light source badge */
.light-badge {
  position: absolute;
  top: -4px;
  left: -4px;
  width: 18px;
  height: 18px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.light-badge svg {
  width: 12px;
  height: 12px;
  fill: #666;
}

.light-badge.light-active svg {
  fill: #fbbf24;
  filter: drop-shadow(0 0 2px #fbbf24);
}

/* Glow effect on token with active light */
.token-has-light {
  box-shadow:
    0 2px 4px rgba(0, 0, 0, 0.4),
    0 0 12px 4px rgba(251, 191, 36, 0.4),
    0 0 24px 8px rgba(251, 191, 36, 0.2);
}

.token-has-light:hover {
  box-shadow:
    0 4px 8px rgba(0, 0, 0, 0.5),
    0 0 16px 6px rgba(251, 191, 36, 0.5),
    0 0 32px 12px rgba(251, 191, 36, 0.25);
}

.token-has-light.token-selected {
  box-shadow:
    0 0 0 3px white,
    0 0 0 6px var(--color-primary-500, #3b82f6),
    0 0 16px 8px rgba(251, 191, 36, 0.4);
}

/* Dead token styling */
.token-dead {
  opacity: 0.6;
  filter: grayscale(0.5);
}

.token-dead::after {
  content: '';
  position: absolute;
  inset: 0;
  background: repeating-linear-gradient(
    45deg,
    transparent,
    transparent 4px,
    rgba(139, 0, 0, 0.3) 4px,
    rgba(139, 0, 0, 0.3) 8px
  );
  border-radius: 50%;
  pointer-events: none;
}

.dead-badge {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dead-badge svg {
  width: 70%;
  height: 70%;
  fill: #dc2626;
  filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.5));
}
</style>
