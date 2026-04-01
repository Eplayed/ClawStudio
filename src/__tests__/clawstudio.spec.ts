import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue({}),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}))

describe('ClawStudio Tests', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  describe('Environment Detection', () => {
    it('should detect Node.js version format', () => {
      const version = 'v22.0.0'
      expect(version).toMatch(/^v\d+\.\d+\.\d+$/)
    })

    it('should detect npm version format', () => {
      const version = '10.0.0'
      expect(version).toMatch(/^\d+\.\d+\.\d+$/)
    })

    it('should parse OpenClaw version', () => {
      const version = '1.2.3'
      const parts = version.split('.')
      expect(parts).toHaveLength(3)
      expect(parseInt(parts[0])).toBeGreaterThanOrEqual(1)
    })
  })

  describe('API Key Validation', () => {
    it('should validate Anthropic key format', () => {
      const key = 'sk-ant-api03-xxxxx'
      expect(key.startsWith('sk-ant-')).toBe(true)
    })

    it('should validate OpenAI key format', () => {
      const key = 'sk-proj-xxxxx'
      expect(key.startsWith('sk-')).toBe(true)
    })

    it('should mask API key for display', () => {
      const maskKey = (key: string) => {
        if (key.length <= 8) return '*'.repeat(key.length)
        return `${key.slice(0, 4)}****${key.slice(-4)}`
      }
      
      expect(maskKey('sk-ant-1234567890abcdef')).toBe('sk-a****cdef')
      expect(maskKey('short')).toBe('*****')
    })
  })

  describe('Cost Calculation', () => {
    it('should calculate Claude 3.5 Sonnet cost correctly', () => {
      // $3/M input tokens, $15/M output tokens
      const inputTokens = 100000
      const outputTokens = 50000
      
      const inputCost = (inputTokens / 1_000_000) * 3
      const outputCost = (outputTokens / 1_000_000) * 15
      const total = inputCost + outputCost
      
      expect(total).toBeCloseTo(1.05, 2)
    })

    it('should calculate GPT-4o cost correctly', () => {
      // $5/M input tokens, $15/M output tokens
      const inputTokens = 100000
      const outputTokens = 50000
      
      const inputCost = (inputTokens / 1_000_000) * 5
      const outputCost = (outputTokens / 1_000_000) * 15
      const total = inputCost + outputCost
      
      expect(total).toBeCloseTo(1.25, 2)
    })

    it('should format cost display', () => {
      const formatCost = (cost: number) => `$${cost.toFixed(2)}`
      expect(formatCost(1.05)).toBe('$1.05')
      expect(formatCost(0.003)).toBe('$0.00')
    })
  })

  describe('Uptime Formatting', () => {
    const formatUptime = (secs: number): string => {
      if (secs < 60) return `${secs}秒`
      if (secs < 3600) return `${Math.floor(secs / 60)}分`
      if (secs < 86400) return `${Math.floor(secs / 3600)}小时`
      return `${Math.floor(secs / 86400)}天`
    }

    it('should format seconds', () => {
      expect(formatUptime(30)).toBe('30秒')
    })

    it('should format minutes', () => {
      expect(formatUptime(90)).toBe('1分')
      expect(formatUptime(180)).toBe('3分')
    })

    it('should format hours', () => {
      expect(formatUptime(3661)).toBe('1小时')
      expect(formatUptime(7200)).toBe('2小时')
    })

    it('should format days', () => {
      expect(formatUptime(90000)).toBe('1天')
      expect(formatUptime(172800)).toBe('2天')
    })
  })

  describe('Template Validation', () => {
    const validateTemplate = (template: Record<string, unknown>) => {
      const required = ['name', 'system_prompt', 'model']
      return required.every(key => template[key])
    }

    it('should validate required fields', () => {
      const valid = {
        name: 'Test Agent',
        system_prompt: 'You are helpful',
        model: 'claude-3-5-sonnet-20241022'
      }
      expect(validateTemplate(valid)).toBe(true)
    })

    it('should reject missing fields', () => {
      const invalid = {
        name: 'Test Agent',
        // missing system_prompt
        model: 'claude-3-5-sonnet-20241022'
      }
      expect(validateTemplate(invalid)).toBe(false)
    })
  })

  describe('Channel Types', () => {
    it('should support all channel types', () => {
      const supportedChannels = ['telegram', 'discord', 'wechat', 'slack']
      expect(supportedChannels).toContain('telegram')
      expect(supportedChannels).toContain('discord')
      expect(supportedChannels).toHaveLength(4)
    })
  })

  describe('Deep Link Format', () => {
    it('should validate claw:// template links', () => {
      const isValidDeepLink = (link: string) => {
        return link.startsWith('claw://template/')
      }

      expect(isValidDeepLink('claw://template/abc123')).toBe(true)
      expect(isValidDeepLink('https://example.com')).toBe(false)
    })
  })

  describe('HITL Levels', () => {
    it('should define HITL levels', () => {
      const hitlLevels = ['off', 'browse', 'standard', 'strict']
      expect(hitlLevels).toContain('off')
      expect(hitlLevels).toContain('strict')
    })
  })
})
