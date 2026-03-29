// computer_use.ts - Computer Use API client and types
// Frontend interface for CU runtime

import { invoke } from '@tauri-apps/api/core'

export interface CUSession {
  id: string
  agent_id: string
  sandbox_id: string
  vnc_port: number
  model: string
  system_prompt: string
  status: 'running' | 'paused' | 'completed' | 'failed'
  messages: CUMessage[]
  total_steps: number
  total_cost: number
  created_at: string
}

export interface CUMessage {
  role: 'user' | 'assistant'
  content: CUContentBlock[]
}

export type CUContentBlock =
  | { type: 'text'; text: string }
  | { type: 'tool_use'; id: string; name: string; input: Record<string, unknown> }
  | { type: 'tool_result'; tool_use_id: string; content: string }
  | { type: 'image'; source: ImageSource }
  | { type: 'thinking'; thinking: string }

export interface ImageSource {
  type: 'base64'
  media_type: 'image/png' | 'image/jpeg'
  data: string
}

export interface CUStepResult {
  session_id: string
  step: number
  response: CUMessage
  paused: boolean
  cost: number
}

export interface HITLRequest {
  session_id: string
  tool_name: string
  action_description: string
  screenshot?: string
  timeout_sec: number
}

// ─── API Client ───

export class CUClient {
  /**
   * Start a new Computer Use session
   */
  static async startSession(
    agentId: string,
    sandboxId: string,
    vncPort: number,
    systemPrompt: string,
    model: string,
    apiKey: string
  ): Promise<string> {
    return invoke<string>('start_cu_session', {
      agent_id: agentId,
      sandbox_id: sandboxId,
      vnc_port: vncPort,
      system_prompt: systemPrompt,
      model,
      api_key: apiKey,
    })
  }

  /**
   * Execute one step of the tool loop
   */
  static async step(
    sessionId: string,
    apiKey: string,
    permLevel: string
  ): Promise<CUStepResult> {
    return invoke<CUStepResult>('cu_step', {
      session_id: sessionId,
      api_key: apiKey,
      perm_level: permLevel,
    })
  }

  /**
   * Pause session for HITL approval
   */
  static async pause(sessionId: string): Promise<void> {
    return invoke<void>('pause_cu_session', {
      session_id: sessionId,
    })
  }

  /**
   * Resume session after HITL approval
   */
  static async resume(sessionId: string, userResponse: string): Promise<void> {
    return invoke<void>('resume_cu_session', {
      session_id: sessionId,
      user_response: userResponse,
    })
  }

  /**
   * Stop session
   */
  static async stop(sessionId: string): Promise<void> {
    return invoke<void>('stop_cu_session', {
      session_id: sessionId,
    })
  }
}

// ─── Utilities ───

export function formatCUAction(tool: string, input: Record<string, unknown>): string {
  switch (tool) {
    case 'computer':
      const action = input.action as string
      switch (action) {
        case 'screenshot':
          return '📷 Screenshot'
        case 'mouse_move':
          return `🖱 Mouse move to (${input.x}, ${input.y})`
        case 'left_click':
          return `🖱 Left click at (${input.x}, ${input.y})`
        case 'right_click':
          return `🖱 Right click at (${input.x}, ${input.y})`
        case 'double_click':
          return `🖱 Double click at (${input.x}, ${input.y})`
        case 'type':
          return `⌨ Type: "${input.text}"`
        case 'key':
          return `⌨ Press key: ${input.key}`
        case 'scroll':
          return `🖱 Scroll ${input.direction} at (${input.x}, ${input.y})`
        default:
          return `🖱 ${action}`
      }
    case 'text_editor':
      const cmd = input.command as string
      switch (cmd) {
        case 'view':
          return `📄 View file: ${input.path}`
        case 'str_replace':
          return `✏️ Edit file: ${input.path}`
        case 'create':
          return `📝 Create file: ${input.path}`
        default:
          return `📄 ${cmd}`
      }
    case 'bash':
      return `💻 Execute: ${(input.command as string).slice(0, 50)}`
    default:
      return `🔧 ${tool}`
  }
}

export function extractScreenshot(message: CUMessage): string | null {
  for (const block of message.content) {
    if (block.type === 'image') {
      return block.source.data
    }
  }
  return null
}

export function extractText(message: CUMessage): string {
  const texts: string[] = []
  for (const block of message.content) {
    if (block.type === 'text') {
      texts.push(block.text)
    } else if (block.type === 'thinking') {
      texts.push(`💭 ${block.thinking}`)
    }
  }
  return texts.join('\n')
}
