/**
 * TokenRenderer: the combat marks on tokens (MIMIR-T-0680).
 */
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import TokenRenderer from '@/components/tokens/TokenRenderer.vue'
import type { Token } from '@/types/api'

function token(id: string): Token {
  return {
    id,
    map_id: 'map-1',
    name: id,
    token_type: 'monster',
    size: 'medium',
    x: 35,
    y: 35,
    visible_to_players: true,
    color: null,
    image_path: null,
    monster_id: 'mm-1',
    character_id: null,
    notes: null,
    vision_type: 'normal',
    vision_range_ft: null,
    vision_bright_ft: null,
    vision_dim_ft: null,
    vision_dark_ft: 0,
    light_radius_ft: 0,
    created_at: '',
    updated_at: '',
  }
}

describe('TokenRenderer combat marks', () => {
  it('marks the current turn, the selection and the down tokens', () => {
    const wrapper = mount(TokenRenderer, {
      props: {
        tokens: [token('a'), token('b'), token('c')],
        gridSizePx: 70,
        currentTurnTokenId: 'a',
        selectedTokenId: 'b',
        deadTokenIds: ['c'],
      },
    })
    const [a, b, c] = wrapper.findAll('.token')
    expect(a.classes()).toContain('token-current-turn')
    expect(b.classes()).toContain('token-selected')
    expect(b.classes()).not.toContain('token-current-turn')
    expect(c.classes()).toContain('token-dead')
  })
})
