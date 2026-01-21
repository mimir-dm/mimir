<template>
  <div class="tool-confirmation">
    <div class="confirmation-header">
      <span class="action-indicator">🔧</span>
      <h4>{{ confirmation.request.action.title }}</h4>
    </div>
    
    <p class="confirmation-description">
      {{ confirmation.request.action.description }}
    </p>
    
    <div class="changes-list">
      <!-- File Edit Changes -->
      <div v-if="confirmation.request.action.changes.type === 'FileEdit'" class="file-edit-changes">
        <div class="file-info">
          <span class="file-path">📄 {{ confirmation.request.action.changes.file_path }}</span>
          <span class="edit-summary">
            {{ confirmation.request.action.changes.edits.length }} edit(s) affecting 
            {{ confirmation.request.action.changes.total_lines_affected }} line(s) 
            of {{ confirmation.request.action.changes.total_lines_in_file }} total
          </span>
        </div>
        
        <div class="edit-list">
          <div 
            v-for="(edit, index) in confirmation.request.action.changes.edits" 
            :key="index" 
            class="edit-item"
          >
            <div class="edit-header">
              <span class="operation-badge" :class="`operation-${edit.operation}`">
                {{ edit.operation.toUpperCase() }}
              </span>
              <span class="line-range">
                Lines {{ edit.start_line }}{{ edit.end_line !== edit.start_line ? `-${edit.end_line}` : '' }}
              </span>
            </div>
            
            <div class="edit-content">
              <!-- Context before -->
              <div 
                v-for="(line, idx) in edit.context_before" 
                :key="`before-${idx}`" 
                class="context-line"
              >
                <span class="line-number">{{ edit.start_line - edit.context_before.length + idx }}</span>
                <span class="line-prefix context">  </span>
                <span class="line-content context">{{ line }}</span>
              </div>
              
              <!-- Old content (for replace/delete) -->
              <div 
                v-for="(line, idx) in edit.old_content" 
                :key="`old-${idx}`" 
                class="content-line removed"
              >
                <span class="line-number">{{ edit.start_line + idx }}</span>
                <span class="line-prefix removed">- </span>
                <span class="line-content">{{ line || '(empty line)' }}</span>
              </div>
              
              <!-- New content (for replace/insert) -->
              <div 
                v-for="(line, idx) in edit.new_content" 
                :key="`new-${idx}`" 
                class="content-line added"
              >
                <span class="line-number">{{ edit.start_line + idx }}</span>
                <span class="line-prefix added">+ </span>
                <span class="line-content">{{ line || '(empty line)' }}</span>
              </div>
              
              <!-- Context after -->
              <div 
                v-for="(line, idx) in edit.context_after" 
                :key="`after-${idx}`" 
                class="context-line"
              >
                <span class="line-number">{{ edit.end_line + 1 + idx }}</span>
                <span class="line-prefix context">  </span>
                <span class="line-content context">{{ line }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- File Write Changes -->
      <div v-else-if="confirmation.request.action.changes.type === 'FileWrite'" class="file-write-changes">
        <div class="file-info">
          <span class="file-path">📄 {{ confirmation.request.action.changes.file_path }}</span>
          <span class="write-info">{{ confirmation.request.action.changes.content_length }} characters</span>
        </div>
        
        <div v-if="confirmation.request.action.changes.diff_preview" class="diff-preview">
          <div class="diff-stats">
            <span class="added">+{{ confirmation.request.action.changes.diff_preview.added_lines }}</span>
            <span class="removed">-{{ confirmation.request.action.changes.diff_preview.removed_lines }}</span>
          </div>
          <pre class="diff-content">{{ confirmation.request.action.changes.diff_preview.preview }}</pre>
        </div>
        
        <!-- Show content preview for new files (when no diff available) -->
        <div v-else-if="confirmation.request.action.changes.content_preview" class="content-preview">
          <div class="preview-header">New file content:</div>
          <pre class="new-file-content">{{ confirmation.request.action.changes.content_preview }}</pre>
        </div>
      </div>

      <!-- Generic Changes (fallback) -->
      <div v-else-if="confirmation.request.action.changes.type === 'Generic'" class="generic-changes">
        <div 
          v-for="(change, index) in confirmation.request.action.changes.items" 
          :key="index" 
          class="change-item"
        >
          <span class="change-bullet">•</span>
          <span class="change-content" :class="{ 'content-block': change.includes('\n') }">
            <pre v-if="change.includes('\n')">{{ change }}</pre>
            <span v-else>{{ change }}</span>
          </span>
        </div>
      </div>
    </div>
    
    <div class="confirmation-buttons" v-if="confirmation.status === 'pending'">
      <button @click="handleApprove" class="btn-confirm" :disabled="isProcessing">
        <span>✓ Approve</span>
      </button>
      <button @click="handleReject" class="btn-reject" :disabled="isProcessing">
        <span>✗ Cancel</span>
      </button>
    </div>
    
    <div v-else class="confirmation-result">
      <span v-if="confirmation.status === 'confirmed'" class="result-confirmed">
        ✓ Action approved and executed
      </span>
      <span v-else class="result-rejected">
        ✗ Action cancelled
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { PendingConfirmation } from '@/stores/chat'

const props = defineProps<{
  confirmation: PendingConfirmation
}>()

const emit = defineEmits<{
  confirm: [id: string]
  reject: [id: string]
}>()

const isProcessing = ref(false)

// Removed risk level logic - all confirmations are treated equally

const handleApprove = async (event: Event) => {
  event.preventDefault()
  event.stopPropagation()
  
  if (isProcessing.value) return
  isProcessing.value = true
  try {
    emit('confirm', props.confirmation.request.id)
    // Don't reset isProcessing here - keep button disabled after click
  } catch (error) {
    console.error('Error confirming action:', error)
    isProcessing.value = false
  }
}

const handleReject = async (event: Event) => {
  event.preventDefault()
  event.stopPropagation()
  
  if (isProcessing.value) return
  isProcessing.value = true
  try {
    emit('reject', props.confirmation.request.id)
    // Don't reset isProcessing here - keep button disabled after click
  } catch (error) {
    console.error('Error rejecting action:', error)
    isProcessing.value = false
  }
}
</script>

<style scoped>
.tool-confirmation {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  margin: 8px 0;
  width: 100%;
}

.tool-confirmation {
  border-left: 3px solid var(--color-primary-500);
}

.confirmation-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.confirmation-header h4 {
  margin: 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--color-text-primary);
}

.action-indicator {
  font-size: 1.2rem;
  flex-shrink: 0;
}

.confirmation-description {
  color: var(--color-text-secondary);
  margin-bottom: 12px;
  line-height: 1.5;
}

.changes-list {
  background: var(--color-surface);
  border-radius: 4px;
  padding: 12px;
  margin-bottom: 16px;
  width: 100%;
  box-sizing: border-box;
}

.change-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  color: var(--color-text-secondary);
  font-size: 0.9rem;
  border-bottom: 1px solid var(--color-border-light, rgba(255, 255, 255, 0.05));
}

.change-item:last-child {
  border-bottom: none;
}

.change-bullet {
  flex-shrink: 0;
  margin-right: 4px;
  color: var(--color-text-tertiary);
  margin-top: 2px;
}

.change-content {
  flex: 1;
  word-break: break-word;
}

.change-content pre {
  display: block !important;
  margin: 8px 0;
  padding: 16px;
  background: var(--color-surface-darker, rgba(0, 0, 0, 0.3));
  border: 1px solid var(--color-border, rgba(255, 255, 255, 0.1));
  border-radius: 6px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 0.85rem;
  line-height: 1.6;
  max-height: 600px;
  overflow-y: auto;
  width: 100%;
  box-sizing: border-box;
  min-height: 100px;
}

.confirmation-buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-confirm,
.btn-reject {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 0.95rem;
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 100px;
  user-select: none;
}

.btn-confirm {
  background: var(--color-success);
  color: white;
}

.btn-confirm:hover:not(:disabled) {
  background: var(--color-success-hover);
}

.btn-reject {
  background: var(--color-surface-elevated);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

.btn-reject:hover:not(:disabled) {
  background: var(--color-surface-hover);
}

.btn-confirm:disabled,
.btn-reject:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirmation-result {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 12px;
  border-radius: 4px;
  background: var(--color-surface);
}

.result-confirmed,
.result-rejected {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
}

.result-confirmed {
  color: var(--color-success);
}

.result-rejected {
  color: var(--color-text-tertiary);
}

/* File editing styles */
.file-edit-changes, .file-write-changes {
  width: 100%;
}

.file-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding: 12px;
  background: var(--color-surface-darker, rgba(0, 0, 0, 0.2));
  border-radius: 6px;
  border: 1px solid var(--color-border);
}

.file-path {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 0.9rem;
  color: var(--color-text-primary);
  font-weight: 600;
}

.edit-summary, .write-info {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-weight: normal;
}

.edit-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.edit-item {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.edit-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 12px;
  background: var(--color-surface-darker, rgba(0, 0, 0, 0.2));
  border-bottom: 1px solid var(--color-border);
}

.operation-badge {
  font-size: 0.75rem;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.operation-badge.operation-replace {
  background: var(--color-warning, #f59e0b);
  color: var(--color-surface);
}

.operation-badge.operation-insert {
  background: var(--color-success, #10b981);
  color: var(--color-surface);
}

.operation-badge.operation-delete {
  background: var(--color-error, #ef4444);
  color: var(--color-surface);
}

.line-range {
  font-size: 0.85rem;
  color: var(--color-text-secondary);
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
}

.edit-content {
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 0.85rem;
  line-height: 1.5;
}

.content-line, .context-line {
  display: flex;
  align-items: flex-start;
  min-height: 20px;
}

.line-number {
  flex-shrink: 0;
  width: 50px;
  text-align: right;
  padding-right: 10px;
  color: var(--color-text-tertiary);
  font-size: 0.8rem;
  font-weight: 500;
  user-select: none;
  border-right: 1px solid var(--color-border-light, rgba(255, 255, 255, 0.1));
  margin-right: 8px;
}

.line-prefix {
  flex-shrink: 0;
  width: 24px;
  text-align: center;
  font-weight: 600;
  user-select: none;
}

.line-prefix.context {
  color: var(--color-text-tertiary);
}

.line-prefix.removed {
  color: var(--color-error, #ef4444);
  background: rgba(239, 68, 68, 0.1);
}

.line-prefix.added {
  color: var(--color-success, #10b981);
  background: rgba(16, 185, 129, 0.1);
}

.line-content {
  flex: 1;
  padding: 2px 8px;
  word-break: break-all;
  white-space: pre-wrap;
}

.line-content.context {
  color: var(--color-text-tertiary);
}

.content-line.removed {
  background: rgba(239, 68, 68, 0.03);
}

.content-line.added {
  background: rgba(16, 185, 129, 0.03);
}

.context-line {
  background: var(--color-surface-elevated);
}

/* Diff preview styles */
.diff-preview {
  margin-top: 12px;
}

.diff-stats {
  display: flex;
  gap: 12px;
  margin-bottom: 8px;
  font-size: 0.85rem;
  font-weight: 600;
}

.diff-stats .added {
  color: var(--color-success, #10b981);
}

.diff-stats .removed {
  color: var(--color-error, #ef4444);
}

.diff-content {
  background: var(--color-surface-darker, rgba(0, 0, 0, 0.3)) !important;
  border: 1px solid var(--color-border) !important;
  padding: 12px !important;
  border-radius: 6px !important;
  font-size: 0.82rem !important;
  line-height: 1.5 !important;
  max-height: 400px;
  overflow-y: auto;
}

/* Content preview styles */
.content-preview {
  margin-top: 12px;
}

.preview-header {
  font-size: 0.85rem;
  font-weight: 600;
  margin-bottom: 8px;
  color: var(--color-text-secondary);
}

.new-file-content {
  background: var(--color-surface-darker, rgba(0, 0, 0, 0.3)) !important;
  border: 1px solid var(--color-success, #10b981) !important;
  border-left: 3px solid var(--color-success, #10b981) !important;
  padding: 12px !important;
  border-radius: 6px !important;
  font-size: 0.82rem !important;
  line-height: 1.5 !important;
  max-height: 300px;
  overflow-y: auto;
  font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
}
</style>