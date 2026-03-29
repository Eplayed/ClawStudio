// eventParser.ts - OpenClaw event stream parser
// Parses raw OpenClaw JSON output into structured UI events

export type EventType = 'think' | 'action' | 'observe' | 'error' | 'unknown'

export interface ToolParams {
  x?: number
  y?: number
  text?: string
  button?: string
  key?: string
  selector?: string
  url?: string
  [key: string]: unknown
}

export interface ParsedEvent {
  type: EventType
  timestamp: string
  tool?: string
  params?: ToolParams
  text?: string
  screenshot?: string // base64
  tokenUsage?: {
    input: number
    output: number
    image: number
  }
  raw: string
  step?: number
}

interface RawEvent {
  type?: string
  tool_use?: Array<{
    name?: string
    input?: Record<string, unknown>
  }>
  tool_result?: unknown
  text?: string
  content?: string | Array<unknown>
  reasoning?: string
  thinking?: string
  screenshot?: string
  token_usage?: {
    input?: number
    output?: number
    image?: number
  }
}

/**
 * Parse a raw OpenClaw event string into a structured ParsedEvent.
 * Handles multiple OpenClaw output formats.
 */
export function parseOpenClawEvent(raw: string): ParsedEvent {
  const timestamp = new Date().toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })

  let parsed: RawEvent = {}
  try {
    parsed = JSON.parse(raw)
  } catch {
    return {
      type: 'error',
      timestamp,
      text: raw.slice(0, 200),
      raw,
    }
  }

  // Format: { type: "thinking" } or { thinking: "..." }
  if (parsed.type === 'thinking' || parsed.thinking || parsed.reasoning) {
    return {
      type: 'think',
      timestamp,
      text: (parsed.thinking || parsed.reasoning || parsed.text || '') as string,
      raw,
    }
  }

  // Format: { type: "tool_use", tool_use: [{ name: "mouse_move", input: { x, y } }] }
  if (parsed.tool_use && Array.isArray(parsed.tool_use) && parsed.tool_use.length > 0) {
    const tool = parsed.tool_use[0]
    return {
      type: 'action',
      timestamp,
      tool: tool.name || 'unknown',
      params: (tool.input || {}) as ToolParams,
      text: tool.name
        ? `${tool.name}(${formatParams(tool.input || {})})`
        : undefined,
      raw,
    }
  }

  // Format: { type: "tool_result" } or { tool_result: "..." }
  if (parsed.type === 'tool_result' || parsed.tool_result !== undefined) {
    let text = ''
    const result = parsed.tool_result ?? parsed.content
    if (typeof result === 'string') {
      text = result.slice(0, 150)
    } else if (Array.isArray(result)) {
      text = `[${result.length} items]`
    } else if (result !== null && result !== undefined) {
      text = JSON.stringify(result).slice(0, 150)
    }
    return {
      type: 'observe',
      timestamp,
      text,
      raw,
    }
  }

  // Format: { type: "error" } or generic error
  if (parsed.type === 'error') {
    return {
      type: 'error',
      timestamp,
      text: (parsed.text || parsed.content || 'Unknown error') as string,
      raw,
    }
  }

  // Format: { screenshot: "base64..." }
  if (parsed.screenshot) {
    return {
      type: 'observe',
      timestamp,
      screenshot: parsed.screenshot,
      text: '[Screenshot captured]',
      raw,
    }
  }

  // Format: { token_usage: { input, output, image } }
  if (parsed.token_usage) {
    const tu = parsed.token_usage
    return {
      type: 'observe',
      timestamp,
      tokenUsage: {
        input: tu.input || 0,
        output: tu.output || 0,
        image: tu.image || 0,
      },
      text: `Token usage: ${tu.input || 0} in / ${tu.output || 0} out / ${tu.image || 0} img`,
      raw,
    }
  }

  // Fallback: treat text as observe
  if (parsed.text || parsed.content) {
    return {
      type: 'observe',
      timestamp,
      text: (parsed.text || parsed.content || '') as string,
      raw,
    }
  }

  return {
    type: 'unknown',
    timestamp,
    text: raw.slice(0, 200),
    raw,
  }
}

function formatParams(params: Record<string, unknown>): string {
  return Object.entries(params)
    .slice(0, 3)
    .map(([k, v]) => `${k}:${typeof v === 'string' ? v.slice(0, 20) : JSON.stringify(v)}`)
    .join(', ')
}

/**
 * Extract cursor position from an action event
 */
export function extractCursorPosition(event: ParsedEvent): { x: number; y: number } | null {
  if (event.type !== 'action') return null
  const x = event.params?.x
  const y = event.params?.y
  if (x !== undefined && y !== undefined) {
    return { x: Number(x), y: Number(y) }
  }
  return null
}

/**
 * Highlight tool calls in log content for display
 */
export function highlightToolCalls(text: string): string {
  if (!text) return ''
  // Match patterns like mouse_move(x:342, y:120) or type_text(text:"hello")
  return text.replace(
    /([a-z_]+)\s*\(\s*([^)]+)\s*\)/gi,
    '<span class="highlight">$1</span>(<span class="warn-hl">$2</span>)'
  )
}
