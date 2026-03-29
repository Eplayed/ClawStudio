// computer_use_store.ts - Computer Use session management
// Manages CU sessions and integrates with event stream

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { CUClient, type CUSession, type CUStepResult, type CUMessage } from '@/utils/computer_use'

export const useCUStore = defineStore('computer_use', () => {
  const sessions = ref<Map<string, CUSession>>(new Map())
  const activeSessions = ref<string[]>([])
  const pausedSessions = ref<Map<string, string>>(new Map()) // session_id -> reason

  // ─── Computed ───

  const activeSession = computed(() => {
    if (activeSessions.value.length === 0) return null
    const id = activeSessions.value[0]
    return sessions.value.get(id) || null
  })

  const totalCost = computed(() => {
    let total = 0
    for (const session of sessions.value.values()) {
      total += session.total_cost
    }
    return total
  })

  // ─── Session Management ───

  async function startSession(
    agentId: string,
    sandboxId: string,
    vncPort: number,
    systemPrompt: string,
    model: string,
    apiKey: string
  ): Promise<string> {
    const sessionId = await CUClient.startSession(
      agentId,
      sandboxId,
      vncPort,
      systemPrompt,
      model,
      apiKey
    )

    // Create local session object
    const session: CUSession = {
      id: sessionId,
      agent_id: agentId,
      sandbox_id: sandboxId,
      vnc_port: vncPort,
      model,
      system_prompt: systemPrompt,
      status: 'running',
      messages: [],
      total_steps: 0,
      total_cost: 0,
      created_at: new Date().toISOString(),
    }

    sessions.value.set(sessionId, session)
    activeSessions.value.push(sessionId)

    return sessionId
  }

  async function step(
    sessionId: string,
    apiKey: string,
    permLevel: string
  ): Promise<CUStepResult> {
    const result = await CUClient.step(sessionId, apiKey, permLevel)

    // Update local session
    const session = sessions.value.get(sessionId)
    if (session) {
      session.messages.push(result.response)
      session.total_steps = result.step
      session.total_cost += result.cost

      if (result.paused) {
        session.status = 'paused'
        pausedSessions.value.set(sessionId, 'Awaiting HITL approval')
      }
    }

    return result
  }

  async function pause(sessionId: string, reason: string = 'User paused'): Promise<void> {
    await CUClient.pause(sessionId)
    const session = sessions.value.get(sessionId)
    if (session) {
      session.status = 'paused'
      pausedSessions.value.set(sessionId, reason)
    }
  }

  async function resume(sessionId: string, userResponse: string): Promise<void> {
    await CUClient.resume(sessionId, userResponse)
    const session = sessions.value.get(sessionId)
    if (session) {
      session.status = 'running'
      pausedSessions.value.delete(sessionId)
    }
  }

  async function stop(sessionId: string): Promise<void> {
    await CUClient.stop(sessionId)
    const session = sessions.value.get(sessionId)
    if (session) {
      session.status = 'completed'
      activeSessions.value = activeSessions.value.filter((id) => id !== sessionId)
    }
  }

  function getSession(sessionId: string): CUSession | undefined {
    return sessions.value.get(sessionId)
  }

  function getAllSessions(): CUSession[] {
    return Array.from(sessions.value.values())
  }

  return {
    sessions,
    activeSessions,
    pausedSessions,
    activeSession,
    totalCost,
    startSession,
    step,
    pause,
    resume,
    stop,
    getSession,
    getAllSessions,
  }
})
