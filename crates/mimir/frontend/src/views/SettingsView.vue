<template>
  <MainLayout>
    <div class="settings-view">
      <h1 class="page-title">Settings</h1>
      
      <div class="settings-layout">
        <!-- Sidebar Navigation -->
        <nav class="settings-sidebar">
          <div class="sidebar-section">
            <h3 class="sidebar-section-title">Admin Tools</h3>
            <ul class="sidebar-nav">
              <li>
                <button 
                  @click="activeSection = 'manage-campaigns'"
                  :class="['nav-item', { active: activeSection === 'manage-campaigns' }]"
                >
                  Manage Campaigns
                </button>
              </li>
              <li>
                <button
                  @click="activeSection = 'import-books'"
                  :class="['nav-item', { active: activeSection === 'import-books' }]"
                >
                  Import Books
                </button>
              </li>
            </ul>
          </div>

          <div class="sidebar-section">
            <h3 class="sidebar-section-title">Application</h3>
            <ul class="sidebar-nav">
              <li>
                <button
                  @click="activeSection = 'theme'"
                  :class="['nav-item', { active: activeSection === 'theme' }]"
                >
                  Theme
                </button>
              </li>
              <li>
                <button
                  @click="activeSection = 'about'"
                  :class="['nav-item', { active: activeSection === 'about' }]"
                >
                  About
                </button>
              </li>
            </ul>
          </div>

          <!-- Developer Tools (dev mode only) -->
          <div v-if="isDevMode" class="sidebar-section">
            <h3 class="sidebar-section-title">Developer</h3>
            <ul class="sidebar-nav">
              <li>
                <button
                  @click="activeSection = 'dev-tools'"
                  :class="['nav-item', { active: activeSection === 'dev-tools' }]"
                >
                  Dev Tools
                </button>
              </li>
            </ul>
          </div>
        </nav>
        
        <!-- Content Area -->
        <main class="settings-content">
          <!-- Theme -->
          <div v-if="activeSection === 'theme'" class="content-section">
            <h2 class="content-title">Theme</h2>
            <p class="content-description">Customize the application appearance</p>
            <div class="form-group">
              <ThemeSelector />
            </div>

            <div class="form-divider"></div>

            <h2 class="content-title">Claude Integration</h2>
            <p class="content-description">Connect Mimir with Claude Code or Claude Desktop for AI-assisted campaign management</p>

            <!-- MCP Integration Instructions -->
            <div class="mcp-integration-section">
              <h3 class="integration-title">Connect Claude to Mimir</h3>
              <p class="integration-description">
                Use Claude Code or Claude Desktop to manage your campaigns with AI assistance.
                Claude handles starting the MCP server automatically.
              </p>

              <!-- Database Path Info -->
              <div class="integration-method">
                <h4 class="method-title">Your Database Path</h4>
                <p class="method-description">This is where Mimir stores your campaign data:</p>
                <div class="code-block">
                  <code>{{ databasePath || 'Loading...' }}</code>
                  <button
                    v-if="databasePath"
                    @click="copyToClipboard(databasePath)"
                    class="copy-button"
                    title="Copy to clipboard"
                  >
                    {{ copiedText === databasePath ? 'Copied!' : 'Copy' }}
                  </button>
                </div>
              </div>

              <!-- Claude Code CLI -->
              <div class="integration-method">
                <h4 class="method-title">Claude Code (CLI)</h4>
                <p class="method-description">Run this command to add Mimir to Claude Code:</p>
                <div class="code-block code-block-multiline">
                  <pre>{{ claudeCodeCommand }}</pre>
                  <button
                    @click="copyToClipboard(claudeCodeCommand)"
                    class="copy-button"
                    title="Copy to clipboard"
                  >
                    {{ copiedText === claudeCodeCommand ? 'Copied!' : 'Copy' }}
                  </button>
                </div>
              </div>

              <!-- Claude Desktop -->
              <div class="integration-method">
                <h4 class="method-title">Claude Desktop</h4>
                <p class="method-description">
                  Add this to your Claude Desktop config
                  (<code>~/Library/Application Support/Claude/claude_desktop_config.json</code>):
                </p>
                <div class="code-block code-block-multiline">
                  <pre>{{ claudeDesktopConfig }}</pre>
                  <button
                    @click="copyToClipboard(claudeDesktopConfig)"
                    class="copy-button"
                    title="Copy to clipboard"
                  >
                    {{ copiedText === claudeDesktopConfig ? 'Copied!' : 'Copy' }}
                  </button>
                </div>
              </div>
            </div>
          </div>

          <!-- About -->
          <div v-else-if="activeSection === 'about'" class="content-section">
            <h2 class="content-title">About Mimir</h2>
            <p class="content-description">Application information</p>
            <div class="about-info">
              <div class="info-row">
                <span class="info-label">Version</span>
                <span class="info-value">{{ appVersion || 'Loading...' }}</span>
              </div>
            </div>
          </div>

          <!-- Dev Tools (dev mode only) -->
          <div v-else-if="activeSection === 'dev-tools' && isDevMode" class="content-section">
            <h2 class="content-title">Developer Tools</h2>
            <p class="content-description">Development-only tools for testing and debugging</p>

            <div class="dev-tools-card">
              <h3 class="card-title">Test Data Seeder</h3>
              <p class="card-description">
                Seed the database with "The Lost Mine of Phandelver" test campaign including modules,
                characters, monsters, and NPCs for development testing.
              </p>

              <div class="seed-status">
                <span class="status-label">Status:</span>
                <span :class="['status-badge', isDevSeeded ? 'seeded' : 'not-seeded']">
                  {{ isDevSeeded ? 'Test Data Present' : 'Not Seeded' }}
                </span>
              </div>

              <div class="seed-actions">
                <button
                  v-if="!isDevSeeded"
                  @click="handleSeedData"
                  class="button button-primary"
                  :disabled="seedActionPending"
                >
                  {{ seedActionPending ? 'Seeding...' : 'Seed Test Data' }}
                </button>
                <button
                  v-if="isDevSeeded"
                  @click="handleReseedData"
                  class="button button-secondary"
                  :disabled="seedActionPending"
                >
                  {{ seedActionPending ? 'Reseeding...' : 'Reseed (Clear & Recreate)' }}
                </button>
                <button
                  v-if="isDevSeeded"
                  @click="handleClearData"
                  class="button button-danger"
                  :disabled="seedActionPending"
                >
                  {{ seedActionPending ? 'Clearing...' : 'Clear Test Data' }}
                </button>
              </div>

              <p v-if="seedMessage" :class="['seed-message', seedMessageType]">
                {{ seedMessage }}
              </p>
            </div>

            <div class="dev-info-card">
              <h3 class="card-title">Prerequisites</h3>
              <ul class="prereq-list">
                <li>Import the Monster Manual (MM) via Library for full monster data display</li>
                <li>Seed assets (maps) are bundled with dev builds</li>
              </ul>
            </div>
          </div>

        </main>
      </div>
    </div>
    
    <!-- Book Management Modal -->
    <BookManagementModal 
      :visible="showBookManagementModal"
      @close="handleBookModalClose"
    />
    
    <!-- Campaign Management Modal -->
    <CampaignManagementModal 
      :visible="showCampaignManagementModal"
      @close="handleCampaignModalClose"
    />
  </MainLayout>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import MainLayout from '../shared/components/layout/MainLayout.vue'
import ThemeSelector from '../shared/components/ui/ThemeSelector.vue'
import BookManagementModal from '@/components/dialogs/BookManagementModal.vue'
import CampaignManagementModal from '@/components/dialogs/CampaignManagementModal.vue'

const showBookManagementModal = ref(false)
const showCampaignManagementModal = ref(false)
const activeSection = ref('theme')
const appVersion = ref('')

// MCP Integration state
const copiedText = ref('')
const databasePath = ref('')

// Dev Tools state
const isDevMode = ref(false)
const isDevSeeded = ref(false)
const seedActionPending = ref(false)
const seedMessage = ref('')
const seedMessageType = ref<'success' | 'error'>('success')

// Computed Claude Code CLI command
const claudeCodeCommand = computed(() => {
  const dbPath = databasePath.value || '/path/to/mimir.db'
  return `claude mcp add mimir \\
  -e MIMIR_DATABASE_PATH="${dbPath}" \\
  -- mimir-mcp`
})

// Computed Claude Desktop config JSON
const claudeDesktopConfig = computed(() => {
  return JSON.stringify({
    mcpServers: {
      mimir: {
        command: "mimir-mcp",
        args: [],
        env: {
          MIMIR_DATABASE_PATH: databasePath.value || "/path/to/mimir.db"
        }
      }
    }
  }, null, 2)
})

// Load app info on mount
onMounted(async () => {
  // Check dev mode and seeded status
  await checkDevMode()

  // Load app info for MCP integration
  try {
    interface AppInfo {
      database_path: string
      app_dir: string
      config_dir: string
      data_dir: string
      resources_dir: string | null
    }
    const appInfoResponse = await invoke<{ success: boolean; data: AppInfo }>('get_app_info')
    if (appInfoResponse.success && appInfoResponse.data) {
      databasePath.value = appInfoResponse.data.database_path
    }
  } catch (error) {
    console.error('Failed to load app info:', error)
  }

  try {
    appVersion.value = await getVersion()
  } catch (error) {
    console.error('Failed to get app version:', error)
    appVersion.value = 'Unknown'
  }
})

// Dev Tools functions
async function checkDevMode() {
  try {
    const response = await invoke<{ success: boolean; data: boolean }>('is_dev_mode')
    isDevMode.value = response.success && response.data
    if (isDevMode.value) {
      await checkDevSeeded()
    }
  } catch (error) {
    console.error('Failed to check dev mode:', error)
    isDevMode.value = false
  }
}

async function checkDevSeeded() {
  try {
    const response = await invoke<{ success: boolean; data: boolean }>('is_dev_seeded')
    isDevSeeded.value = response.success && response.data
  } catch (error) {
    console.error('Failed to check dev seeded status:', error)
    isDevSeeded.value = false
  }
}

async function handleSeedData() {
  seedActionPending.value = true
  seedMessage.value = ''
  try {
    const response = await invoke<{ success: boolean; data: boolean; error?: string }>('seed_dev_data')
    if (response.success) {
      seedMessage.value = response.data
        ? 'Test data seeded successfully!'
        : 'Test data already exists.'
      seedMessageType.value = 'success'
      await checkDevSeeded()
    } else {
      seedMessage.value = response.error || 'Failed to seed data'
      seedMessageType.value = 'error'
    }
  } catch (error) {
    seedMessage.value = `Error: ${error}`
    seedMessageType.value = 'error'
  } finally {
    seedActionPending.value = false
  }
}

async function handleReseedData() {
  seedActionPending.value = true
  seedMessage.value = ''
  try {
    const response = await invoke<{ success: boolean; data: boolean; error?: string }>('reseed_dev_data')
    if (response.success) {
      seedMessage.value = 'Test data reseeded successfully!'
      seedMessageType.value = 'success'
      await checkDevSeeded()
    } else {
      seedMessage.value = response.error || 'Failed to reseed data'
      seedMessageType.value = 'error'
    }
  } catch (error) {
    seedMessage.value = `Error: ${error}`
    seedMessageType.value = 'error'
  } finally {
    seedActionPending.value = false
  }
}

async function handleClearData() {
  seedActionPending.value = true
  seedMessage.value = ''
  try {
    const response = await invoke<{ success: boolean; error?: string }>('clear_dev_data')
    if (response.success) {
      seedMessage.value = 'Test data cleared successfully!'
      seedMessageType.value = 'success'
      await checkDevSeeded()
    } else {
      seedMessage.value = response.error || 'Failed to clear data'
      seedMessageType.value = 'error'
    }
  } catch (error) {
    seedMessage.value = `Error: ${error}`
    seedMessageType.value = 'error'
  } finally {
    seedActionPending.value = false
  }
}

// MCP Integration methods
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    copiedText.value = text
    setTimeout(() => {
      copiedText.value = ''
    }, 2000)
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
  }
}

// Open modals based on section selection
watch(activeSection, (newSection) => {
  if (newSection === 'import-books') {
    showBookManagementModal.value = true
  } else if (newSection === 'manage-campaigns') {
    showCampaignManagementModal.value = true
  }
})

// When modals close, switch to a different section (theme)
const handleBookModalClose = () => {
  showBookManagementModal.value = false
  activeSection.value = 'theme'
}

const handleCampaignModalClose = () => {
  showCampaignManagementModal.value = false
  activeSection.value = 'theme'
}

</script>

<style scoped>
.settings-view {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-title {
  font-size: 2rem;
  font-weight: 700;
  color: var(--color-text);
  margin-bottom: var(--spacing-xl);
}

.settings-layout {
  flex: 1;
  display: flex;
  gap: var(--spacing-xl);
  min-height: 0;
}

/* Sidebar Navigation */
.settings-sidebar {
  width: 240px;
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  height: fit-content;
  flex-shrink: 0;
}

.sidebar-section {
  margin-bottom: var(--spacing-xl);
}

.sidebar-section:last-child {
  margin-bottom: 0;
}

.sidebar-section-title {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: var(--spacing-md);
}

.sidebar-nav {
  list-style: none;
  margin: 0;
  padding: 0;
}

.sidebar-nav li {
  margin-bottom: var(--spacing-xs);
}

.nav-item {
  display: block;
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  background: none;
  border: none;
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-weight: 500;
  text-align: left;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.nav-item:hover {
  background: var(--color-gray-100);
  color: var(--color-text);
}

.nav-item.active {
  background: var(--color-primary-100);
  color: var(--color-primary-700);
}

.theme-dark .nav-item:hover {
  background: var(--color-gray-700);
}

.theme-dark .nav-item.active {
  background: var(--color-primary-900);
  color: var(--color-primary-300);
}

/* Content Area */
.settings-content {
  flex: 1;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
  min-height: 0;
  overflow-y: auto;
}

.content-section {
  max-width: 1200px;
  width: 100%;
}

.content-title {
  font-size: 1.5rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.content-description {
  font-size: 1rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-xl);
  line-height: 1.5;
}

.action-button {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-md) var(--spacing-lg);
  background: var(--color-primary-500);
  color: white;
  border: none;
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: background-color var(--transition-fast);
}

.action-button:hover {
  background: var(--color-primary-600);
}

/* Form Elements */
.form-group {
  margin-bottom: var(--spacing-lg);
}

.form-label {
  display: block;
  font-weight: 500;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.form-input {
  width: 100%;
  padding: var(--spacing-sm) var(--spacing-md);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  transition: border-color var(--transition-fast);
}

.form-input:focus {
  outline: none;
  border-color: var(--color-primary-500);
}

.input-group {
  display: flex;
  gap: var(--spacing-sm);
}

.button {
  padding: var(--spacing-sm) var(--spacing-lg);
  border-radius: var(--radius-md);
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
  border: none;
}

.button-secondary {
  background-color: var(--color-surface-variant);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.button-secondary:hover {
  background-color: var(--color-gray-200);
  border-color: var(--color-border-hover);
}

.theme-dark .button-secondary:hover {
  background-color: var(--color-gray-700);
}

.input-help {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin-top: var(--spacing-sm);
  line-height: 1.4;
}

.input-help code {
  background-color: var(--color-gray-100);
  padding: 0.125rem 0.25rem;
  border-radius: 0.25rem;
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-size: 0.8125rem;
}

.theme-dark .input-help code {
  background-color: var(--color-gray-700);
}

.input-help a {
  color: var(--color-primary-500);
  text-decoration: underline;
}

.input-help a:hover {
  color: var(--color-primary-600);
}

.form-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-top: var(--spacing-xl);
}

.settings-message {
  font-size: 0.875rem;
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  margin: 0;
}

.settings-message.success {
  background-color: var(--color-success-100);
  color: var(--color-success-700);
  border: 1px solid var(--color-success-300);
}

.settings-message.error {
  background-color: var(--color-error-100);
  color: var(--color-error-700);
  border: 1px solid var(--color-error-300);
}

.theme-dark .settings-message.success {
  background-color: var(--color-success-900);
  color: var(--color-success-300);
  border-color: var(--color-success-700);
}

.theme-dark .settings-message.error {
  background-color: var(--color-error-900);
  color: var(--color-error-300);
  border-color: var(--color-error-700);
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

select.form-input {
  cursor: pointer;
}

/* About Section */
.about-info {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) 0;
}

.info-label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.info-value {
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  color: var(--color-text);
}

/* Form Divider */
.form-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--spacing-xl) 0;
}

/* Toggle Option */
.toggle-option {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.toggle-option:hover {
  border-color: var(--color-primary-300);
}

.toggle-option input[type="checkbox"] {
  width: 18px;
  height: 18px;
  margin-top: 2px;
  cursor: pointer;
  accent-color: var(--color-primary-500);
}

.toggle-content {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.toggle-label {
  font-weight: 500;
  color: var(--color-text);
}

.toggle-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

/* MCP Integration Section */
.mcp-integration-section {
  margin-top: var(--spacing-xl);
  padding: var(--spacing-lg);
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.integration-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.integration-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-lg);
}

.integration-method {
  margin-bottom: var(--spacing-lg);
  padding-bottom: var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.integration-method:last-child {
  margin-bottom: 0;
  padding-bottom: 0;
  border-bottom: none;
}

.method-title {
  font-size: 0.9375rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-xs);
}

.method-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-sm);
}

.method-description code {
  background-color: var(--color-gray-100);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-size: 0.8125rem;
}

.theme-dark .method-description code {
  background-color: var(--color-gray-700);
}

.code-block {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: var(--color-gray-900);
  border-radius: var(--radius-md);
  padding: var(--spacing-sm) var(--spacing-md);
  overflow-x: auto;
}

.code-block code {
  flex: 1;
  color: var(--color-gray-100);
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-size: 0.8125rem;
  white-space: nowrap;
}

.code-block-multiline {
  flex-direction: column;
  align-items: stretch;
}

.code-block-multiline pre {
  flex: 1;
  color: var(--color-gray-100);
  font-family: ui-monospace, 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
  font-size: 0.75rem;
  line-height: 1.5;
  margin: 0;
  white-space: pre;
  overflow-x: auto;
}

.code-block-multiline .copy-button {
  align-self: flex-end;
  margin-top: var(--spacing-sm);
}

.copy-button {
  flex-shrink: 0;
  padding: var(--spacing-xs) var(--spacing-sm);
  background: var(--color-gray-700);
  color: var(--color-gray-200);
  border: none;
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.copy-button:hover {
  background: var(--color-gray-600);
  color: white;
}

/* Dev Tools Styles */
.dev-tools-card,
.dev-info-card {
  background: var(--color-surface-variant);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
}

.card-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--color-text);
  margin-bottom: var(--spacing-sm);
}

.card-description {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
  margin-bottom: var(--spacing-lg);
  line-height: 1.5;
}

.seed-status {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-lg);
}

.seed-status .status-label {
  font-weight: 500;
  color: var(--color-text-secondary);
}

.seed-status .status-badge {
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-sm);
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.seed-status .status-badge.seeded {
  background-color: var(--color-success-100);
  color: var(--color-success-700);
  border: 1px solid var(--color-success-300);
}

.seed-status .status-badge.not-seeded {
  background-color: var(--color-gray-100);
  color: var(--color-gray-600);
  border: 1px solid var(--color-gray-300);
}

.theme-dark .seed-status .status-badge.seeded {
  background-color: var(--color-success-900);
  color: var(--color-success-300);
  border-color: var(--color-success-700);
}

.theme-dark .seed-status .status-badge.not-seeded {
  background-color: var(--color-gray-800);
  color: var(--color-gray-400);
  border-color: var(--color-gray-600);
}

.seed-actions {
  display: flex;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.button-primary {
  background-color: var(--color-primary-500);
  color: white;
  border: none;
}

.button-primary:hover {
  background-color: var(--color-primary-600);
}

.button-danger {
  background-color: var(--color-error-500);
  color: white;
  border: none;
}

.button-danger:hover {
  background-color: var(--color-error-600);
}

.seed-message {
  margin-top: var(--spacing-md);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  font-size: 0.875rem;
}

.seed-message.success {
  background-color: var(--color-success-100);
  color: var(--color-success-700);
  border: 1px solid var(--color-success-300);
}

.seed-message.error {
  background-color: var(--color-error-100);
  color: var(--color-error-700);
  border: 1px solid var(--color-error-300);
}

.theme-dark .seed-message.success {
  background-color: var(--color-success-900);
  color: var(--color-success-300);
  border-color: var(--color-success-700);
}

.theme-dark .seed-message.error {
  background-color: var(--color-error-900);
  color: var(--color-error-300);
  border-color: var(--color-error-700);
}

.prereq-list {
  margin: 0;
  padding-left: var(--spacing-lg);
  color: var(--color-text-secondary);
  font-size: 0.875rem;
  line-height: 1.6;
}

.prereq-list li {
  margin-bottom: var(--spacing-xs);
}
</style>