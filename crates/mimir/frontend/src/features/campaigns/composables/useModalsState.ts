import { ref, type Ref } from 'vue'

/**
 * Modal names for the ModulesTab component
 * Uses exact naming from the original component for seamless integration
 */
export type ModalName =
  | 'createModal'      // Create module modal
  | 'deleteModule'     // Delete module confirmation
  | 'mapUpload'        // Map upload modal
  | 'tokenSetup'       // Token setup modal
  | 'createDoc'        // Create document modal
  | 'deleteDoc'        // Delete document confirmation
  | 'npcSelector'      // NPC selector modal
  | 'exportDialog'     // Export/PDF dialog
  | 'npcDetail'        // NPC detail modal
  | 'monsterEdit'      // Monster customization modal

/**
 * Composable for managing multiple modal states
 * Consolidates all modal visibility refs into a single interface
 */
export function useModalsState() {
  // All modal visibility states (names match original ModulesTab.vue refs)
  const showCreateModal = ref(false)
  const showDeleteModuleModal = ref(false)
  const showMapUploadModal = ref(false)
  const showTokenSetupModal = ref(false)
  const showCreateDocModal = ref(false)
  const showDeleteDocModal = ref(false)
  const showNpcSelector = ref(false)
  const showExportDialog = ref(false)
  const showNpcDetailModal = ref(false)
  const showMonsterEditModal = ref(false)

  // Map of modal names to refs for programmatic access
  const modals: Record<ModalName, Ref<boolean>> = {
    createModal: showCreateModal,
    deleteModule: showDeleteModuleModal,
    mapUpload: showMapUploadModal,
    tokenSetup: showTokenSetupModal,
    createDoc: showCreateDocModal,
    deleteDoc: showDeleteDocModal,
    npcSelector: showNpcSelector,
    exportDialog: showExportDialog,
    npcDetail: showNpcDetailModal,
    monsterEdit: showMonsterEditModal
  }

  /**
   * Open a modal by name
   */
  function openModal(name: ModalName) {
    modals[name].value = true
  }

  /**
   * Close a modal by name
   */
  function closeModal(name: ModalName) {
    modals[name].value = false
  }

  /**
   * Toggle a modal by name
   */
  function toggleModal(name: ModalName) {
    modals[name].value = !modals[name].value
  }

  /**
   * Close all modals
   */
  function closeAllModals() {
    Object.values(modals).forEach(modal => {
      modal.value = false
    })
  }

  /**
   * Check if any modal is open
   */
  function isAnyModalOpen(): boolean {
    return Object.values(modals).some(modal => modal.value)
  }

  return {
    // Individual refs for direct binding compatibility (exact names from ModulesTab.vue)
    showCreateModal,
    showDeleteModuleModal,
    showMapUploadModal,
    showTokenSetupModal,
    showCreateDocModal,
    showDeleteDocModal,
    showNpcSelector,
    showExportDialog,
    showNpcDetailModal,
    showMonsterEditModal,

    // Programmatic access
    openModal,
    closeModal,
    toggleModal,
    closeAllModals,
    isAnyModalOpen
  }
}
