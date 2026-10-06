import type { Theme } from '../types/api'

/**
 * The themes the app ships. This is the only list: no backend command
 * provides themes (MIMIR-T-0668). Each id has a stylesheet in
 * assets/styles/themes/<id>.css and a body class `theme-<id>`.
 */
export const THEMES: readonly Theme[] = [
  { id: 'light', name: 'Light', description: 'Clean light theme with soft purples' },
  { id: 'dark', name: 'Dark', description: 'Deep blues and navy tones' },
  { id: 'hyper', name: 'Hyper', description: 'Vaporwave neon aesthetic' },
]
