import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

/**
 * Configuration for external windows
 */
export interface WindowConfig {
  label: string
  url: string
  title: string
  width?: number
  height?: number
  minWidth?: number
  minHeight?: number
  x?: number
  y?: number
  center?: boolean
}

/**
 * Predefined window configurations
 */
export const WINDOW_CONFIGS = {
  sourcesReference: {
    label: 'sources-reference',
    url: 'sources.html',
    title: 'Source Library',
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    center: false,
    x: 100,
    y: 100
  },
  // Future windows can be added here
  // characterSheet: { ... },
  // diceRoller: { ... },
} as const

/**
 * Opens an external window or focuses it if already open
 * @param config Window configuration or a key from WINDOW_CONFIGS
 */
export async function openExternalWindow(
  config: WindowConfig | keyof typeof WINDOW_CONFIGS
): Promise<void> {
  // If string key provided, get config from predefined configs
  const windowConfig: WindowConfig = 
    typeof config === 'string' ? WINDOW_CONFIGS[config] : config
  
  // Check if window already exists
  const existingWindow = await WebviewWindow.getByLabel(windowConfig.label)
  
  if (existingWindow) {
    // Window exists, bring it to focus
    await existingWindow.setFocus()
  } else {
    // Create new window with provided config
    const windowOptions: any = {
      url: windowConfig.url,
      title: windowConfig.title,
      width: windowConfig.width ?? 1200,
      height: windowConfig.height ?? 800,
      minWidth: windowConfig.minWidth ?? 800,
      minHeight: windowConfig.minHeight ?? 600,
      center: windowConfig.center ?? false,
    }
    
    // Add position if specified
    if (windowConfig.x !== undefined) windowOptions.x = windowConfig.x
    if (windowConfig.y !== undefined) windowOptions.y = windowConfig.y
    
    const newWindow = new WebviewWindow(windowConfig.label, windowOptions)
  }
}

/**
 * Convenience function to open Sources Reference window
 */
export async function openSourcesReference() {
  return openExternalWindow('sourcesReference')
}