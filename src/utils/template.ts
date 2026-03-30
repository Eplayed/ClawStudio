// template.ts - Agent Template Management
// Export, import, and share agent templates

export interface AgentTemplate {
  schema: string
  name: string
  description: string
  author: string
  version: string
  created_at: string
  system_prompt: string
  model: string
  computer_use: boolean
  sandbox_image?: string
  hitl_level: 'browse' | 'standard' | 'auto'
  tags: string[]
  channels: string[]
  max_tokens?: number
  temperature?: number
  budget_limit?: number
  screenshot?: string
}

export interface TemplateMeta {
  id: string
  name: string
  author: string
  version: string
  description: string
  tags: string[]
  verified: boolean
  downloads: number
  rating: number
}

// ─── API Client ───

import { invoke } from '@tauri-apps/api/core'

export class TemplateManager {
  /**
   * Export agent config as a shareable template
   */
  static async exportTemplate(config: {
    name: string
    description: string
    system_prompt: string
    model: string
    computer_use: boolean
    sandbox_image?: string
    hitl_level: string
    tags: string[]
    channels: string[]
    max_tokens?: number
    temperature?: number
    budget_limit?: number
  }): Promise<string> {
    return invoke<string>('export_template', config)
  }

  /**
   * Export template to a file
   */
  static async exportToFile(template: AgentTemplate, path: string): Promise<string> {
    return invoke<string>('export_template_file', { template, path })
  }

  /**
   * Import template from deep link or base64 data
   */
  static async importFromData(data: string): Promise<AgentTemplate> {
    return invoke<AgentTemplate>('import_template', { data })
  }

  /**
   * Import template from file
   */
  static async importFromFile(path: string): Promise<AgentTemplate> {
    return invoke<AgentTemplate>('import_template_file', { path })
  }

  /**
   * Generate a shareable deep link
   */
  static async generateShareLink(template: AgentTemplate): Promise<string> {
    return invoke<string>('generate_share_link', { template })
  }

  /**
   * Validate template for issues
   */
  static async validate(template: AgentTemplate): Promise<string[]> {
    return invoke<string[]>('validate_template', { template })
  }

  /**
   * Get built-in templates
   */
  static async getBuiltinTemplates(): Promise<TemplateMeta[]> {
    return invoke<TemplateMeta[]>('list_builtin_templates')
  }

  /**
   * Get a specific built-in template
   */
  static async getBuiltinTemplate(name: string): Promise<AgentTemplate> {
    return invoke<AgentTemplate>('get_builtin_template', { name })
  }
}

// ─── Utilities ───

/**
 * Download template as a .claw-template file
 */
export function downloadTemplate(template: AgentTemplate): void {
  const json = JSON.stringify(template, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  
  const a = document.createElement('a')
  a.href = url
  a.download = `${template.name.replace(/\s+/g, '-').toLowerCase()}.claw-template`
  a.click()
  
  URL.revokeObjectURL(url)
}

/**
 * Copy deep link to clipboard
 */
export async function copyShareLink(template: AgentTemplate): Promise<void> {
  const link = await TemplateManager.generateShareLink(template)
  await navigator.clipboard.writeText(link)
}

/**
 * Parse template from drag-drop event
 */
export async function parseDroppedFile(file: File): Promise<AgentTemplate> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string
        
        // Try parsing as JSON directly
        if (content.startsWith('{')) {
          const template = JSON.parse(content) as AgentTemplate
          resolve(template)
        } else {
          // Try as base64
          const template = await TemplateManager.importFromData(content)
          resolve(template)
        }
      } catch (error) {
        reject(new Error(`Failed to parse template: ${error}`))
      }
    }
    
    reader.onerror = () => reject(new Error('Failed to read file'))
    reader.readAsText(file)
  })
}

/**
 * Check if URL is a claw:// deep link
 */
export function isClawDeepLink(url: string): boolean {
  return url.startsWith('claw://template/')
}

/**
 * Extract template from deep link
 */
export function extractFromDeepLink(url: string): string | null {
  if (!isClawDeepLink(url)) return null
  return url.replace('claw://template/', '')
}

/**
 * Format template for display
 */
export function formatTemplateSummary(template: AgentTemplate): string {
  const parts: string[] = []
  
  if (template.computer_use) {
    parts.push('🖥 Computer Use')
  }
  
  if (template.channels.length > 0) {
    parts.push(`📡 ${template.channels.join(', ')}`)
  }
  
  parts.push(`🔐 ${template.hitl_level}`)
  
  return parts.join(' • ')
}

/**
 * Get model display name
 */
export function getModelDisplayName(model: string): string {
  const names: Record<string, string> = {
    'claude-3-5-sonnet-20241022': 'Claude 3.5 Sonnet',
    'claude-3-opus-20240229': 'Claude 3 Opus',
    'claude-3-haiku-20240307': 'Claude 3 Haiku',
    'gpt-4o': 'GPT-4o',
    'gpt-4-turbo': 'GPT-4 Turbo',
    'deepseek-chat': 'DeepSeek Chat',
  }
  
  return names[model] || model
}

/**
 * Get HITL level description
 */
export function getHitlDescription(level: string): string {
  const descriptions: Record<string, string> = {
    browse: 'Read-only access, all actions require approval',
    standard: 'Normal access, dangerous actions require approval',
    auto: 'Full automation, only blocked actions require approval',
  }
  
  return descriptions[level] || level
}