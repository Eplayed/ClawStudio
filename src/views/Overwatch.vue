<script setup lang="ts">
/**
 * Overwatch.vue - 监控舱（核心页面）
 * 左侧思维流 + 右侧视觉流 + 底部 HITL 审批栏
 * 支持模拟模式和真实 VNC 连接
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { useAgentStore } from '@/stores/agents'
import ThoughtLog from '@/components/ThoughtLog.vue'
import VisualStream from '@/components/VisualStream.vue'
import HITLBar from '@/components/HITLBar.vue'
import VncConnectModal from '@/components/VncConnectModal.vue'

const route = useRoute()
const agentStore = useAgentStore()

// Active agent
const agentId = ref((route.params.agentId as string) || 'agent-01')
const agent = ref(agentStore.agents.find(a => a.id === agentId.value))

// Visual stream state
const isPlaying = ref(true)
const useRealVnc = ref(false)
const vncUrl = ref('')
const vncPort = ref(6080)
const crosshairX = ref(72)
const crosshairY = ref(78)
const currentCost = ref(agent.value?.currentCost ?? 0)
const budgetLimit = ref(agent.value?.budgetLimit ?? 1.0)
const sandboxName = ref(agent.value?.sandboxId || 'sandbox-ubuntu-01')

// VNC modal state
const showVncModal = ref(false)

// Live cost ticker
let costInterval: ReturnType<typeof setInterval> | undefined

onMounted(async () => {
  // Initialize event listener
  await agentStore.initEventListener()

  // Live cost ticker
  costInterval = setInterval(() => {
    const a = agentStore.agents.find(a => a.id === agentId.value)
    if (a) {
      currentCost.value = a.currentCost
      budgetLimit.value = a.budgetLimit
    } else {
      currentCost.value += 0.001 + Math.random() * 0.003
    }
  }, 3000)
})

onUnmounted(() => {
  if (costInterval) clearInterval(costInterval)
})

// Visual stream controls
function handlePlay() {
  isPlaying.value = true
}

function handlePause() {
  isPlaying.value = false
}

function handleScreenshot() {
  console.log('Screenshot requested')
}

// VNC connection
function handleVncConnect(connected: boolean) {
  console.log('VNC connection status:', connected)
}

function openVncModal() {
  showVncModal.value = true
}

function handleVncUrl(url: string) {
  vncUrl.value = url
  useRealVnc.value = true
  showVncModal.value = false
}

function handleVncDisconnect() {
  useRealVnc.value = false
  vncUrl.value = ''
}

// HITL events
function handleHITLApprove() {
  console.log('HITL: Approved')
}

function handleHITLReject(correction: string) {
  console.log('HITL: Rejected with correction:', correction)
}

function handleHITLTakeover() {
  console.log('HITL: Takeover requested')
}

function handleHITLTimeout() {
  console.log('HITL: Timeout - auto rejected')
}

function handlePermChange(level: string) {
  console.log('Permission level changed to:', level)
}
</script>

<template>
  <div class="page">
    <!-- Top bar -->
    <header class="topbar">
      <div class="page-title">
        📺 监控舱 Overwatch —
        <span class="agent-label">{{ agentId.toUpperCase() }}</span>
        <span class="agent-name" v-if="agent">{{ agent.name }}</span>
      </div>
      <div class="topbar-actions">
        <!-- VNC toggle button -->
        <button
          class="vnc-toggle"
          :class="{ active: useRealVnc }"
          @click="openVncModal"
        >
          🖥 {{ useRealVnc ? 'VNC 已连接' : '连接 VNC' }}
        </button>
        <div class="status-chip">
          <div class="dot"></div>
          实时监控中
        </div>
      </div>
    </header>

    <!-- Main layout -->
    <div class="overwatch-layout">
      <div class="overwatch-main">
        <!-- Left: Thought Stream (40%) -->
        <div class="thought-stream-wrapper">
          <ThoughtLog />
        </div>

        <!-- Right: Visual Stream (60%) -->
        <div class="visual-stream-wrapper">
          <VisualStream
            :sandbox-name="sandboxName"
            :vnc-url="vncUrl"
            :vnc-port="vncPort"
            :use-real-vnc="useRealVnc"
            :is-playing="isPlaying"
            :cursor-x="crosshairX"
            :cursor-y="crosshairY"
            :current-cost="currentCost"
            :budget-limit="budgetLimit"
            @play="handlePlay"
            @pause="handlePause"
            @screenshot="handleScreenshot"
            @vnc-connect="handleVncConnect"
          />
        </div>
      </div>

      <!-- Bottom: HITL Bar -->
      <HITLBar
        :agent-name="agentId"
        @approve="handleHITLApprove"
        @reject="handleHITLReject"
        @takeover="handleHITLTakeover"
        @timeout="handleHITLTimeout"
        @perm-change="handlePermChange"
      />
    </div>

    <!-- VNC Connection Modal -->
    <VncConnectModal
      :visible="showVncModal"
      :current-vnc-url="vncUrl"
      :current-vnc-port="vncPort"
      @close="showVncModal = false"
      @connect="handleVncUrl"
      @disconnect="handleVncDisconnect"
    />
  </div>
</template>

<style scoped>
.page {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.topbar {
  height: 52px;
  min-height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-base);
  flex-shrink: 0;
}

.page-title {
  font-size: 15px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.agent-label {
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 12px;
}

.agent-name {
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 400;
}

.topbar-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.vnc-toggle {
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-secondary);
  font-size: 11px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
}

.vnc-toggle:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.vnc-toggle.active {
  background: var(--green-glow);
  border-color: var(--green-dim);
  color: var(--green);
}

.status-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 500;
  background: var(--green-glow);
  color: var(--green);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--green);
  animation: pulse-dot 2s infinite;
}

@keyframes pulse-dot {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.7; }
}

.overwatch-layout {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.overwatch-main {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.thought-stream-wrapper {
  width: 40%;
  min-width: 340px;
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.visual-stream-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
