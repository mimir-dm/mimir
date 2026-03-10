/**
 * Tests for tokenSizes.ts
 *
 * Tests the token size normalization and conversion utilities.
 */

import { describe, it, expect } from 'vitest'
import { normalizeSize, sizeToTokenSize, SIZE_TO_TOKEN_SIZE } from '@/constants/tokenSizes'

describe('normalizeSize', () => {
  it('converts abbreviated sizes to lowercase', () => {
    expect(normalizeSize('T')).toBe('t')
    expect(normalizeSize('S')).toBe('s')
    expect(normalizeSize('M')).toBe('m')
    expect(normalizeSize('L')).toBe('l')
    expect(normalizeSize('H')).toBe('h')
    expect(normalizeSize('G')).toBe('g')
  })

  it('converts full names to lowercase', () => {
    expect(normalizeSize('Tiny')).toBe('tiny')
    expect(normalizeSize('Medium')).toBe('medium')
    expect(normalizeSize('Gargantuan')).toBe('gargantuan')
  })

  it('takes first element from array', () => {
    expect(normalizeSize(['L'])).toBe('l')
    expect(normalizeSize(['M', 'L'])).toBe('m')
  })

  it('returns m for falsy values', () => {
    expect(normalizeSize(null)).toBe('m')
    expect(normalizeSize(undefined)).toBe('m')
    expect(normalizeSize('')).toBe('m')
    expect(normalizeSize(0)).toBe('m')
  })
})

describe('sizeToTokenSize', () => {
  it('maps abbreviated sizes to TokenSize', () => {
    expect(sizeToTokenSize('t')).toBe('tiny')
    expect(sizeToTokenSize('s')).toBe('small')
    expect(sizeToTokenSize('m')).toBe('medium')
    expect(sizeToTokenSize('l')).toBe('large')
    expect(sizeToTokenSize('h')).toBe('huge')
    expect(sizeToTokenSize('g')).toBe('gargantuan')
  })

  it('maps full names to TokenSize', () => {
    expect(sizeToTokenSize('tiny')).toBe('tiny')
    expect(sizeToTokenSize('large')).toBe('large')
    expect(sizeToTokenSize('gargantuan')).toBe('gargantuan')
  })

  it('handles uppercase input', () => {
    expect(sizeToTokenSize('T')).toBe('tiny')
    expect(sizeToTokenSize('Large')).toBe('large')
  })

  it('handles array input', () => {
    expect(sizeToTokenSize(['L'])).toBe('large')
  })

  it('defaults to medium for unknown sizes', () => {
    expect(sizeToTokenSize('unknown')).toBe('medium')
    expect(sizeToTokenSize(null)).toBe('medium')
    expect(sizeToTokenSize(undefined)).toBe('medium')
  })
})

describe('SIZE_TO_TOKEN_SIZE', () => {
  it('has entries for all abbreviated sizes', () => {
    expect(SIZE_TO_TOKEN_SIZE['t']).toBe('tiny')
    expect(SIZE_TO_TOKEN_SIZE['s']).toBe('small')
    expect(SIZE_TO_TOKEN_SIZE['m']).toBe('medium')
    expect(SIZE_TO_TOKEN_SIZE['l']).toBe('large')
    expect(SIZE_TO_TOKEN_SIZE['h']).toBe('huge')
    expect(SIZE_TO_TOKEN_SIZE['g']).toBe('gargantuan')
  })

  it('has entries for all full names', () => {
    expect(SIZE_TO_TOKEN_SIZE['tiny']).toBe('tiny')
    expect(SIZE_TO_TOKEN_SIZE['small']).toBe('small')
    expect(SIZE_TO_TOKEN_SIZE['medium']).toBe('medium')
    expect(SIZE_TO_TOKEN_SIZE['large']).toBe('large')
    expect(SIZE_TO_TOKEN_SIZE['huge']).toBe('huge')
    expect(SIZE_TO_TOKEN_SIZE['gargantuan']).toBe('gargantuan')
  })
})
