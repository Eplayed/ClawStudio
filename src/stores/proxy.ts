import { defineStore } from 'pinia'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

export interface TokenUsageEvent {
  input: number
  output: number
  image: number
  cost: number
  model: string
}

export interface ThinkingEvent {
  text: string
  step: number
  timestamp?: string
}

export interface ActionEvent {
  tool: string
  params: any
  step: number
  timestamp?: string
}

export interface HitlRequest {
  request_id: string
  tool: string
  params: any
}

export interface HitlResponseEvent {
  request_id: string
  approved: boolean
  response: any | null
}

export interface CircuitEvent {
  reason: string
  current_cost: number
  limit: number
}

export interface ProxyState {
  totalCost: number
  inputTokens: number
  outputTokens: number
  imageTokens: number
  thinkingLog: ThinkingEvent[]
  actionLog: ActionEvent[]
  hitlPending: HitlRequest | null
  circuitBroken: boolean
  budgetLimit: number
}

export const useProxyStore = defineStore('proxy', {
  state: (): ProxyState => ({
    totalCost: 0,
    inputTokens: 0,
    outputTokens: 0,
    imageTokens: 0,
    thinkingLog: [],
    actionLog: [],
    hitlPending: null,
    circuitBroken: false,
    budgetLimit: 100.0,
  }),

  actions: {
    async init() {
      // 获取初始状态
      try {
        const status: any = await invoke('get_proxy_status')
        if (status.total_cost) this.totalCost = status.total_cost
        if (status.circuit_broken) this.circuitBroken = status.circuit_broken
      } catch (e) {
        console.error('Failed to get initial proxy status:', e)
      }

      // 监听各种事件
      await listen<TokenUsageEvent>('proxy:token_usage', (e) => {
        this.totalCost += e.payload.cost
        this.inputTokens += e.payload.input
        this.outputTokens += e.payload.output
        this.imageTokens += e.payload.image
      })

      await listen<ThinkingEvent>('proxy:thinking', (e) => {
        this.thinkingLog.push({ ...e.payload, timestamp: new Date().toLocaleTimeString() })
      })

      await listen<ActionEvent>('proxy:action', (e) => {
        this.actionLog.push({ ...e.payload, timestamp: new Date().toLocaleTimeString() })
      })

      await listen<HitlRequest>('proxy:hitl_request', (e) => {
        this.hitlPending = e.payload
      })

      await listen<HitlResponseEvent>('proxy:hitl_response', (e) => {
        if (this.hitlPending?.request_id === e.payload.request_id) {
          this.hitlPending = null
        }
      })

      await listen<CircuitEvent>('proxy:circuit_breaker', (e) => {
        this.circuitBroken = true
        this.totalCost = e.payload.current_cost
      })
    },

    async hitlApprove() {
      if (!this.hitlPending) return
      try {
        await invoke('hitl_approve', { requestId: this.hitlPending.request_id })
        this.hitlPending = null
      } catch (e) {
        console.error('HITL approve failed:', e)
      }
    },

    async hitlReject(correction?: string) {
      if (!this.hitlPending) return
      try {
        await invoke('hitl_reject', { 
          requestId: this.hitlPending.request_id,
          correction: correction || null
        })
        this.hitlPending = null
      } catch (e) {
        console.error('HITL reject failed:', e)
      }
    },

    async resetCost() {
      try {
        await invoke('reset_proxy_cost')
        this.totalCost = 0
        this.circuitBroken = false
      } catch (e) {
        console.error('Reset cost failed:', e)
      }
    },

    async setBudgetLimit(limit: number) {
      try {
        await invoke('set_proxy_budget_limit', { limit })
        this.budgetLimit = limit
      } catch (e) {
        console.error('Set budget limit failed:', e)
      }
    }
  }
})