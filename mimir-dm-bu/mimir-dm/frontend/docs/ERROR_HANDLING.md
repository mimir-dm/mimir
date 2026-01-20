# Error Handling Guidelines

This document establishes consistent error handling patterns across the frontend application, organized by architectural layer.

## Philosophy

Error handling follows a **catch at the boundary, propagate with context** approach:

1. **Service Layer** - Add operation context to backend errors
2. **Composable Layer** - Propagate errors (minimal handling)
3. **Component Layer** - Catch and display user-friendly messages
4. **Store Layer** - Throw on critical operations, log on non-critical

## Error Flow

```
Backend API
    ↓
Service Layer (adds context: "Failed to update module 123: ...")
    ↓
Composable Layer (propagates, rarely catches)
    ↓
Component Layer (catches and displays to user)
```

---

## 1. Service Layer

**Purpose**: Services are the boundary between frontend and backend. They translate backend errors into meaningful frontend errors.

### Pattern

**Always wrap errors** with operation context using this template:

```typescript
throw new Error(`Failed to ${operation} ${resource}: ${error}`)
```

### Example

```typescript
// src/services/ModuleService.ts
class ModuleService {
  async get(id: number): Promise<Module> {
    try {
      const response = await invoke<ApiResponse<Module>>('get_module', { id })
      if (!response.success) {
        throw new Error(response.error || 'Unknown error')
      }
      return response.data!
    } catch (error) {
      throw new Error(`Failed to get module ${id}: ${error}`)
    }
  }

  async update(id: number, data: UpdateModule): Promise<Module> {
    try {
      const response = await invoke<ApiResponse<Module>>('update_module', {
        id,
        request: data
      })
      if (!response.success) {
        throw new Error(response.error || 'Unknown error')
      }
      return response.data!
    } catch (error) {
      throw new Error(`Failed to update module ${id}: ${error}`)
    }
  }

  async transitionStage(id: number, stage: string): Promise<Module> {
    try {
      const response = await invoke<ApiResponse<Module>>('transition_module_stage', {
        request: { module_id: id, new_stage: stage }
      })
      if (!response.success) {
        throw new Error(response.error || 'Unknown error')
      }
      return response.data!
    } catch (error) {
      throw new Error(`Failed to transition module ${id} to ${stage}: ${error}`)
    }
  }
}
```

### Guidelines

- **Always catch**: Every service method should have a try/catch
- **Add context**: Include operation, resource type, and identifiers
- **Re-throw**: Always throw after adding context (don't swallow)
- **Consistent format**: Use the template for uniformity

### Anti-Patterns

❌ **Don't swallow errors**:
```typescript
async get(id: number) {
  try {
    return await invoke('get_module', { id })
  } catch (error) {
    console.error(error) // Just logging, not re-throwing
    return null // Swallowing the error
  }
}
```

❌ **Don't throw raw errors**:
```typescript
async get(id: number) {
  return await invoke('get_module', { id }) // No context added
}
```

---

## 2. Composable Layer

**Purpose**: Composables encapsulate business logic and state management. They should be thin orchestrators that let service errors propagate.

### Pattern

**Minimal error handling** - let errors propagate to components.

### Example

```typescript
// src/features/modules/composables/useModuleStage.ts
export function useModuleStage(moduleId: Ref<number>) {
  const module = ref<Module | null>(null)

  async function loadModule() {
    // No try/catch - let service error propagate with its context
    module.value = await ModuleService.get(moduleId.value)
  }

  async function transitionToNextStage() {
    const currentStage = module.value?.status
    const nextStage = getNextStage(currentStage)

    // No try/catch - let service error propagate
    await ModuleService.transitionStage(moduleId.value, nextStage)

    // Reload to get updated state
    await loadModule()
  }

  return {
    module,
    loadModule,
    transitionToNextStage
  }
}
```

### When to catch in composables

Only catch errors if you're adding **valuable business context** that the service doesn't have:

```typescript
export function useModuleStage(moduleId: Ref<number>) {
  async function transitionToNextStage() {
    const module = await ModuleService.get(moduleId.value)
    const nextStage = getNextStage(module.status)

    try {
      await ModuleService.transitionStage(moduleId.value, nextStage)
    } catch (error) {
      // Add business context that service doesn't have
      throw new Error(
        `Cannot transition from ${module.status} to ${nextStage}: ${error}`
      )
    }
  }
}
```

### Guidelines

- **Default**: No try/catch in composables
- **Only catch when**: Adding business-level context the service can't provide
- **Always re-throw**: Never swallow errors in composables
- **Keep it thin**: If you're catching many errors, reconsider the abstraction

### Anti-Patterns

❌ **Don't catch just to re-throw**:
```typescript
async function loadModule() {
  try {
    module.value = await ModuleService.get(moduleId.value)
  } catch (error) {
    throw error // Pointless catch
  }
}
```

❌ **Don't catch and set error state** (that's the component's job):
```typescript
const error = ref<string | null>(null)

async function loadModule() {
  try {
    module.value = await ModuleService.get(moduleId.value)
  } catch (err) {
    error.value = err.message // Component should do this
  }
}
```

---

## 3. Component Layer

**Purpose**: Components are the user-facing boundary. They must catch errors and present them in a user-friendly way.

### Pattern

**Always catch errors** from async operations and display them to users.

### Example

```vue
<script setup lang="ts">
import { ref } from 'vue'
import { useModuleStage } from '@/features/modules/composables/useModuleStage'

const props = defineProps<{ moduleId: number }>()

const errorMessage = ref<string | null>(null)
const successMessage = ref<string | null>(null)
const loading = ref(false)

const { module, loadModule, transitionToNextStage } = useModuleStage(
  toRef(props, 'moduleId')
)

async function handleLoadModule() {
  loading.value = true
  errorMessage.value = null

  try {
    await loadModule()
  } catch (error) {
    errorMessage.value = error instanceof Error
      ? error.message
      : 'Failed to load module'
  } finally {
    loading.value = false
  }
}

async function handleTransition() {
  loading.value = true
  errorMessage.value = null
  successMessage.value = null

  try {
    await transitionToNextStage()
    successMessage.value = 'Module transitioned successfully'
  } catch (error) {
    errorMessage.value = error instanceof Error
      ? error.message
      : 'Failed to transition module'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  handleLoadModule()
})
</script>

<template>
  <div>
    <div v-if="errorMessage" class="error-banner">
      {{ errorMessage }}
    </div>

    <div v-if="successMessage" class="success-banner">
      {{ successMessage }}
    </div>

    <button
      @click="handleTransition"
      :disabled="loading"
    >
      Transition to Next Stage
    </button>
  </div>
</template>
```

### Guidelines

- **Always catch**: Every async operation should have error handling
- **Display errors**: Show error messages in the UI (banner, toast, modal)
- **Clear errors**: Reset error state before new operations
- **Loading states**: Manage loading state to prevent duplicate requests
- **Error state**: Use ref for error messages that can be displayed
- **Fallback messages**: Provide fallback if error.message is unavailable

### Error Display Options

1. **Inline error banner** (shown above)
2. **Toast notifications** (for transient errors)
3. **Modal dialogs** (for critical errors requiring acknowledgment)
4. **Form validation errors** (for input-specific errors)

### Anti-Patterns

❌ **Don't let errors go uncaught**:
```vue
<script setup>
async function handleTransition() {
  await transitionToNextStage() // Uncaught promise rejection
}
</script>
```

❌ **Don't just console.log errors**:
```vue
<script setup>
async function handleTransition() {
  try {
    await transitionToNextStage()
  } catch (error) {
    console.error(error) // User never sees this
  }
}
</script>
```

❌ **Don't show technical stack traces to users**:
```vue
<template>
  <div class="error">
    {{ error.stack }} <!-- Too technical -->
  </div>
</template>
```

---

## 4. Store Layer (Pinia)

**Purpose**: Stores manage global application state. Error handling depends on whether the operation is critical to application state.

### Pattern

**Critical operations**: Throw errors (let component handle)
**Non-critical operations**: Log and continue

### Critical Operations Example

Critical operations affect core application state and must succeed:

```typescript
// src/stores/campaigns.ts
export const useCampaignStore = defineStore('campaigns', () => {
  const currentCampaign = ref<Campaign | null>(null)

  async function loadCampaign(id: number) {
    try {
      const response = await invoke<ApiResponse<Campaign>>('get_campaign', { id })
      if (!response.success) {
        throw new Error(response.error || 'Failed to load campaign')
      }
      currentCampaign.value = response.data!
    } catch (error) {
      // Critical operation - must succeed for app to function
      throw new Error(`Failed to load campaign ${id}: ${error}`)
    }
  }

  async function updateCampaign(id: number, data: CampaignUpdate) {
    try {
      const response = await invoke<ApiResponse<Campaign>>('update_campaign', {
        id,
        update: data
      })
      if (!response.success) {
        throw new Error(response.error || 'Failed to update campaign')
      }
      currentCampaign.value = response.data!
    } catch (error) {
      // Critical - state must stay consistent
      throw new Error(`Failed to update campaign: ${error}`)
    }
  }

  return { currentCampaign, loadCampaign, updateCampaign }
})
```

### Non-Critical Operations Example

Non-critical operations enhance UX but aren't required for core functionality:

```typescript
// src/stores/sharedContext.ts
export const useSharedContextStore = defineStore('sharedContext', () => {
  const context = ref<CampaignContext>({})
  const lastSyncTime = ref<Date | null>(null)

  async function syncMetrics() {
    try {
      await invoke('sync_usage_metrics', {
        campaign_id: context.value.campaign?.id
      })
      lastSyncTime.value = new Date()
    } catch (error) {
      // Non-critical - log and continue
      console.warn('Failed to sync metrics (non-critical):', error)
      // Don't throw - app should continue working
    }
  }

  async function logUserAction(action: string) {
    try {
      await invoke('log_user_action', { action })
    } catch (error) {
      // Non-critical analytics - ignore failures
      console.warn('Failed to log user action:', error)
    }
  }

  return { context, syncMetrics, logUserAction }
})
```

### Decision Matrix

| Operation Type | Error Handling | Examples |
|---------------|----------------|----------|
| **Critical** | Throw error | Load campaign, Update module, Create document |
| **Non-Critical** | Log and continue | Analytics, Metrics sync, User activity tracking |

Ask yourself:
- Can the app function if this fails? → Non-critical (log and continue)
- Does this affect core state? → Critical (throw error)
- Is this a nice-to-have feature? → Non-critical (log and continue)

### Guidelines

- **Identify criticality**: Decide if operation is critical before implementing
- **Critical = throw**: Let components handle with user feedback
- **Non-critical = log**: Use console.warn to track but don't disrupt UX
- **Document decisions**: Add comments explaining why operation is critical/non-critical

### Anti-Patterns

❌ **Don't swallow critical errors**:
```typescript
async function loadCampaign(id: number) {
  try {
    currentCampaign.value = await CampaignService.get(id)
  } catch (error) {
    console.error(error) // Should throw, not just log
    currentCampaign.value = null // Leaves app in broken state
  }
}
```

❌ **Don't throw on non-critical operations**:
```typescript
async function syncMetrics() {
  try {
    await invoke('sync_metrics')
  } catch (error) {
    throw error // Unnecessary - metrics aren't critical
  }
}
```

---

## 5. Backend Error Response Structure

Since Phase 2 (MIMIR-T-0035) is complete, backend errors now have a structured format:

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
}

// Error types from backend (src/types.rs):
type ApiErrorType =
  | 'Database'
  | 'Io'
  | 'Serialization'
  | 'NotFound'
  | 'Validation'
  | 'BadRequest'
  | 'PermissionDenied'
  | 'Internal'
```

### Handling Structured Errors

```typescript
async function get(id: number): Promise<Module> {
  try {
    const response = await invoke<ApiResponse<Module>>('get_module', { id })
    if (!response.success) {
      // Backend provides structured error message
      throw new Error(response.error || 'Unknown error')
    }
    return response.data!
  } catch (error) {
    // Add service-level context
    throw new Error(`Failed to get module ${id}: ${error}`)
  }
}
```

---

## Summary

### Quick Reference

| Layer | Catch Errors? | Action | Display to User? |
|-------|--------------|--------|------------------|
| **Service** | Always | Wrap with context and re-throw | No |
| **Composable** | Rarely | Propagate (or add business context) | No |
| **Component** | Always | Show user-friendly message | Yes |
| **Store (Critical)** | Always | Re-throw with context | No (component will) |
| **Store (Non-Critical)** | Always | Log and continue | No |

### Error Message Template

Service layer:
```typescript
`Failed to ${operation} ${resource} ${id}: ${error}`
```

Examples:
- `Failed to get module 123: Database error: connection timeout`
- `Failed to update campaign settings: Validation error: name cannot be empty`
- `Failed to delete document 456: Not found: Document with id '456' not found`

---

## Migration Checklist

When updating existing code:

- [ ] Services: Add try/catch with context wrapping
- [ ] Composables: Remove unnecessary try/catch
- [ ] Components: Ensure all async operations catch errors
- [ ] Stores: Identify critical vs non-critical operations
- [ ] Test error scenarios to verify user sees appropriate messages
