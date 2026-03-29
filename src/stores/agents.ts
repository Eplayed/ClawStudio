// Pinia store - Agent state management
// Bridges Tauri backend commands with Vue reactive state

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { parseOpenClawEvent, extractCursorPosition } from '@/utils/eventParser'
import { calculateCost } from '@/utils/costCalculator'

export interface Agent {
  id: string
  name: string
  avatar: string
  systemPrompt: string
  computerUse: boolean
  networkAccess: boolean
  sandboxId: string | null
  fileWhitelist: string
  budgetLimit: number
  tokenLimit: number
  // Runtime state
  status: 'queued' | 'running' | 'paused' | 'stopped' | 'completed'
  currentCost: number
  tokensUsed: number
  elapsedSec: number
  progress: number
}

export interface OpenClawEvent {
  agent_id: string
  type?: string
  tool?: string
  params?: Record<string, unknown>
  text?: string
  raw?: string
  timestamp?: string
}

export interface CursorPosition {
  x: number
  y: number
  percentX: number
  percentY: number
}

interface QueryPlan {
  sql: string
  params: unknown[]
}

export const useAgentStore = defineStore('agents', () => {
  const agents = ref<Agent[]>([])
  const eventLog = ref<OpenClawEvent[]>([])
  const cursorPosition = ref<CursorPosition | null>(null)

  const runningAgents = computed(() =>
    agents.value.filter((a) => a.status === 'running')
  )
  const totalCostToday = computed(() =>
    agents.value.reduce((sum, a) => sum + a.currentCost, 0)
  )

  // ─── Backend Integration ───

  async function startAgent(agent: Agent) {
    try {
      const _plan = await invoke<QueryPlan>('save_agent_config', {
        agent: {
          id: agent.id,
          name: agent.name,
          avatar: agent.avatar,
          system_prompt: agent.systemPrompt,
          computer_use: agent.computerUse,
          network_access: agent.networkAccess,
          sandbox_id: agent.sandboxId,
          file_whitelist: agent.fileWhitelist,
          budget_limit: agent.budgetLimit,
          token_limit: agent.tokenLimit,
        },
      })

      await invoke('start_agent', {
        agentId: agent.id,
        agentName: agent.name,
        configPath: `/tmp/clawstudio_${agent.id}.yaml`,
      })

      const existing = agents.value.find((a) => a.id === agent.id)
      if (existing) {
        existing.status = 'running'
      } else {
        agent.status = 'running'
        agents.value.push(agent)
      }
    } catch (e) {
      console.error('Failed to start agent:', e)
      throw e
    }
  }

  async function stopAgent(agentId: string) {
    try {
      await invoke('stop_agent', { agentId })
      const agent = agents.value.find((a) => a.id === agentId)
      if (agent) agent.status = 'stopped'
    } catch (e) {
      console.error('Failed to stop agent:', e)
    }
  }

  function addAgent(agent: Agent) {
    const existing = agents.value.find((a) => a.id === agent.id)
    if (!existing) {
      agents.value.push(agent)
    }
  }

  function removeAgent(agentId: string) {
    agents.value = agents.value.filter((a) => a.id !== agentId)
  }

  // ─── Event Stream Listener (B2) ───

  async function initEventListener() {
    // Only listen once
    if (_listenerActive) return
    _listenerActive = true

    await listen<string>('openclaw-event', (event) => {
      try {
        const raw = event.payload as string
        const parsed = parseOpenClawEvent(raw)

        // Cast to OpenClawEvent for store
        const uiEvent: OpenClawEvent = {
          agent_id: parsed.type, // NOTE: type field reused; real impl would carry agent_id
          type: parsed.type,
          tool: parsed.tool,
          params: parsed.params as Record<string, unknown>,
          text: parsed.text,
          raw: parsed.raw,
        }

        eventLog.value.push(uiEvent)

        // Keep log bounded
        if (eventLog.value.length > 500) {
          eventLog.value = eventLog.value.slice(-300)
        }

        // Update cursor position from mouse_move events
        const cursor = extractCursorPosition(parsed)
        if (cursor) {
          cursorPosition.value = {
            x: cursor.x,
            y: cursor.y,
            percentX: Math.min(100, Math.max(0, (cursor.x / 1920) * 100)),
            percentY: Math.min(100, Math.max(0, (cursor.y / 1080) * 100)),
          }
        }

        // Update cost from tokenUsage events
        if (parsed.tokenUsage) {
          const runningAgent = runningAgents.value[0]
          if (runningAgent) {
            const model = 'claude-3-5-sonnet-20241022'
            const cost = calculateCost(model, parsed.tokenUsage)
            runningAgent.currentCost += cost
            runningAgent.tokensUsed +=
              parsed.tokenUsage.input +
              parsed.tokenUsage.output +
              parsed.tokenUsage.image

            // Budget check
            if (
              runningAgent.budgetLimit > 0 &&
              runningAgent.currentCost >= runningAgent.budgetLimit
            ) {
              stopAgent(runningAgent.id)
              console.warn(
                `Agent ${runningAgent.id} budget exceeded: $${runningAgent.currentCost.toFixed(4)}`
              )
            }
          }
        }
      } catch (e) {
        console.warn('Failed to process openclaw event:', e)
      }
    })
  }

  let _listenerActive = false

  return {
    agents,
    eventLog,
    cursorPosition,
    runningAgents,
    totalCostToday,
    startAgent,
    stopAgent,
    addAgent,
    removeAgent,
    initEventListener,
  }
})
