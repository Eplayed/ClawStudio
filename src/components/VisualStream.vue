<script setup lang="ts">
/**
 * VisualStream.vue - 视觉流面板（noVNC 集成版）
 * 支持模拟桌面（默认）和真实 VNC 连接
 */
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
// noVNC - 使用动态导入或在运行时加载
// import NovncPanel from '@novnc/novnc/core/NovncPanel'
// import RFB from '@novnc/novnc/core/rfb'
// 类型声明
declare const RFB: any

// Props
const props = defineProps<{
  sandboxName?: string
  vncPort?: number
  vncUrl?: string
  cursorX?: number
  cursorY?: number
  isPlaying?: boolean
  currentCost?: number
  budgetLimit?: number
  useRealVnc?: boolean
}>()

const emit = defineEmits<{
  play: []
  pause: []
  screenshot: []
  'vnc-connect': [connected: boolean]
}>()

// noVNC state
const vncContainer = ref<HTMLElement | null>(null)
const vncConnected = ref(false)
const vncError = ref('')
let rfbInstance: any = null

// 模拟模式：游标位置
const crossX = ref(props.cursorX ?? 72)
const crossY = ref(props.cursorY ?? 78)

// 模拟模式：包围框
const bboxX = ref(60)
const bboxY = ref(40)
const bboxW = ref(25)
const bboxH = ref(20)

// 模拟游标移动动画
let cursorInterval: ReturnType<typeof setInterval> | undefined
let bboxInterval: ReturnType<typeof setInterval> | undefined

const presets = [
  { x: 72, y: 78 },
  { x: 45, y: 35 },
  { x: 82, y: 55 },
  { x: 30, y: 65 },
  { x: 58, y: 48 },
  { x: 88, y: 30 },
  { x: 15, y: 85 },
  { x: 70, y: 62 },
]

const presetBboxes = [
  { x: 65, y: 72, w: 18, h: 14 },
  { x: 38, y: 28, w: 22, h: 18 },
  { x: 78, y: 50, w: 15, h: 12 },
  { x: 26, y: 60, w: 20, h: 16 },
]

let presetIdx = 0

// 成本显示
const costDisplay = computed(() => (props.currentCost ?? 0).toFixed(3))
const costPercent = computed(() => {
  if (!props.budgetLimit || props.budgetLimit === 0) return 0
  return Math.min(100, ((props.currentCost ?? 0) / props.budgetLimit) * 100)
})
const costColor = computed(() => {
  const pct = costPercent.value
  if (pct >= 80) return 'var(--red)'
  if (pct >= 50) return 'var(--amber)'
  return 'var(--green)'
})

// 初始化 noVNC 连接
function initVnc() {
  if (!props.useRealVnc || !vncContainer.value) return

  const url = props.vncUrl || `ws://localhost:${props.vncPort || 6080}`
  
  try {
    rfbInstance = new RFB(vncContainer.value, url, {
      credentials: { password: '' },
    })

    rfbInstance.addEventListener('connect', () => {
      vncConnected.value = true
      vncError.value = ''
      emit('vnc-connect', true)
      console.log('VNC connected')
    })

    rfbInstance.addEventListener('disconnect', () => {
      vncConnected.value = false
      emit('vnc-connect', false)
      console.log('VNC disconnected')
    })

    rfbInstance.addEventListener('securityfailure', (e: any) => {
      vncError.value = `VNC 认证失败: ${e.detail.reason}`
      console.error('VNC security failure:', e)
    })
  } catch (e) {
    vncError.value = `VNC 连接失败: ${e}`
    console.error('VNC init error:', e)
  }
}

// 断开 VNC
function disconnectVnc() {
  if (rfbInstance) {
    rfbInstance.disconnect()
    rfbInstance = null
    vncConnected.value = false
  }
}

// 启动模拟动画
function startSimulation() {
  if (props.useRealVnc) return

  cursorInterval = window.setInterval(() => {
    if (props.isPlaying !== false) {
      presetIdx = (presetIdx + 1) % presets.length
      crossX.value = presets[presetIdx].x
      crossY.value = presets[presetIdx].y
    }
  }, 2500)

  bboxInterval = window.setInterval(() => {
    if (props.isPlaying !== false) {
      const b = presetBboxes[presetIdx % presetBboxes.length]
      bboxX.value = b.x
      bboxY.value = b.y
      bboxW.value = b.w
      bboxH.value = b.h
    }
  }, 2500)
}

// 停止模拟动画
function stopSimulation() {
  if (cursorInterval) clearInterval(cursorInterval)
  if (bboxInterval) clearInterval(bboxInterval)
}

onMounted(() => {
  if (props.useRealVnc) {
    initVnc()
  } else {
    startSimulation()
  }
})

onUnmounted(() => {
  disconnectVnc()
  stopSimulation()
})

// 切换到真实 VNC 时初始化
watch(() => props.useRealVnc, (useVnc) => {
  if (useVnc) {
    stopSimulation()
    initVnc()
  } else {
    disconnectVnc()
    startSimulation()
  }
})

// 切换 VNC URL 时重连
watch(() => props.vncUrl, () => {
  if (props.useRealVnc) {
    disconnectVnc()
    initVnc()
  }
})

function handlePlay() {
  emit('play')
}

function handlePause() {
  emit('pause')
}

function handleScreenshot() {
  emit('screenshot')
}
</script>

<template>
  <div class="visual-stream">
    <!-- Toolbar -->
    <div class="visual-stream-toolbar">
      <div class="toolbar-left">
        <button
          class="vs-btn"
          :class="{ active: isPlaying }"
          @click="isPlaying ? handlePause() : handlePlay()"
          :title="isPlaying ? '暂停' : '播放'"
        >
          {{ isPlaying ? '⏸' : '▶' }}
        </button>
        <button class="vs-btn" @click="handleScreenshot" title="截图">
          📷
        </button>
        <span class="vs-speed">1.0×</span>
        <span class="sandbox-label">
          📦 {{ sandboxName || 'sandbox-ubuntu-01' }}
          <span v-if="useRealVnc" class="vnc-badge" :class="{ connected: vncConnected }">
            {{ vncConnected ? '已连接' : '未连接' }}
          </span>
        </span>
      </div>
      <div class="toolbar-right">
        <div
          class="cost-live"
          :style="{ color: costColor, borderColor: `color-mix(in srgb, ${costColor} 30%, transparent)`, background: `color-mix(in srgb, ${costColor} 10%, transparent)` }"
        >
          🔥 <span class="cost-val">${{ costDisplay }}</span>
          <span style="opacity:0.6;margin-left:2px">/ ${{ (budgetLimit ?? 1).toFixed(2) }}</span>
        </div>
      </div>
    </div>

    <!-- Screen viewport -->
    <div class="screen-viewport">
      <!-- 真实 VNC 连接模式 -->
      <div v-if="useRealVnc" class="vnc-wrapper">
        <div ref="vncContainer" class="vnc-container"></div>
        <div v-if="vncError" class="vnc-error">
          {{ vncError }}
        </div>
      </div>

      <!-- 模拟桌面模式 -->
      <div v-else class="screen-frame">
        <div class="sim-desktop">
          <!-- Taskbar -->
          <div class="sim-taskbar">
            <div class="tb-dot r"></div>
            <div class="tb-dot y"></div>
            <div class="tb-dot g"></div>
            <div class="tb-title">ubuntu — Firefox</div>
          </div>

          <!-- Browser area -->
          <div class="sim-browser">
            <div class="sim-url-bar">
              <div class="sim-url">https://invoice.example.com/monthly</div>
            </div>
            <div class="sim-page">
              <div class="sim-page-content">
                <div class="sim-heading">📄 月度发票整理系统</div>
                <p>Invoice Management Dashboard — 当前处理中</p>
                <table class="sim-table">
                  <thead>
                    <tr>
                      <th>编号</th>
                      <th>金额</th>
                      <th>日期</th>
                      <th>状态</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>INV-2024-001</td>
                      <td>¥3,450</td>
                      <td>2024-01-05</td>
                      <td style="color:var(--green)">✓ 已归档</td>
                    </tr>
                    <tr>
                      <td>INV-2024-002</td>
                      <td>¥1,280</td>
                      <td>2024-01-12</td>
                      <td style="color:var(--amber)">处理中</td>
                    </tr>
                    <tr>
                      <td>INV-2024-003</td>
                      <td>¥5,900</td>
                      <td>2024-01-18</td>
                      <td style="color:var(--text-dim)">待处理</td>
                    </tr>
                  </tbody>
                </table>
                <div class="sim-btn-row">
                  <button class="sim-btn primary">存档</button>
                  <button class="sim-btn danger">删除</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- AI Bounding Box -->
        <div
          class="ai-bbox"
          :style="{
            left: bboxX + '%',
            top: bboxY + '%',
            width: bboxW + '%',
            height: bboxH + '%',
          }"
        ></div>

        <!-- AI Crosshair -->
        <div
          class="ai-crosshair"
          :style="{ left: crossX + '%', top: crossY + '%' }"
        >
          <div class="crosshair-ring"></div>
          <div class="crosshair-dot"></div>
          <div class="crosshair-line-h"></div>
          <div class="crosshair-line-v"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.visual-stream {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-deep);
}

.visual-stream-toolbar {
  padding: 8px 16px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-base);
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.vs-btn {
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.vs-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.vs-btn.active {
  background: var(--cyan-glow);
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.vs-speed {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-dim);
  padding: 2px 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-card);
}

.sandbox-label {
  font-size: 11px;
  color: var(--text-dim);
  display: flex;
  align-items: center;
  gap: 5px;
}

.vnc-badge {
  font-size: 9px;
  padding: 2px 6px;
  border-radius: 8px;
  background: var(--red-glow);
  color: var(--red);
}

.vnc-badge.connected {
  background: var(--green-glow);
  color: var(--green);
}

.cost-live {
  font-family: var(--font-mono);
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 14px;
  border: 1px solid;
  display: flex;
  align-items: center;
  gap: 4px;
}

.cost-val {
  font-weight: 600;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
}

/* Screen viewport */
.screen-viewport {
  flex: 1;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(ellipse at center, rgba(6, 214, 214, 0.03) 0%, transparent 70%),
    var(--bg-deep);
  padding: 12px;
}

/* VNC wrapper */
.vnc-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.vnc-container {
  width: 100%;
  height: 100%;
  background: #0d0d20;
}

.vnc-error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  padding: 16px 24px;
  background: var(--red-glow);
  border: 1px solid var(--red);
  border-radius: var(--radius);
  color: var(--red);
  font-size: 12px;
}

/* Screen frame (simulation mode) */
.screen-frame {
  width: 92%;
  height: 88%;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  position: relative;
  background: #1a1a2e;
  box-shadow:
    0 0 40px rgba(0, 0, 0, 0.6),
    0 0 80px rgba(6, 214, 214, 0.05);
}

/* Scan line effect */
.screen-frame::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: rgba(6, 214, 214, 0.08);
  animation: scanline 4s linear infinite;
  pointer-events: none;
  z-index: 20;
}

@keyframes scanline {
  from { top: 0; }
  to   { top: 100%; }
}

/* Simulated desktop */
.sim-desktop {
  width: 100%;
  height: 100%;
  background: linear-gradient(135deg, #1e1e3a, #12122a);
  position: relative;
  display: flex;
  flex-direction: column;
}

.sim-taskbar {
  height: 32px;
  background: #0d0d20;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 8px;
  border-bottom: 1px solid #222248;
}

.sim-taskbar .tb-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.sim-taskbar .tb-dot.r { background: #ef4444; }
.sim-taskbar .tb-dot.y { background: #eab308; }
.sim-taskbar .tb-dot.g { background: #22c55e; }

.sim-taskbar .tb-title {
  flex: 1;
  text-align: center;
  font-size: 11px;
  color: var(--text-dim);
  font-family: var(--font-mono);
}

.sim-browser {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.sim-url-bar {
  height: 28px;
  background: #15152e;
  display: flex;
  align-items: center;
  padding: 0 10px;
  border-bottom: 1px solid #222248;
}

.sim-url {
  flex: 1;
  background: #0d0d20;
  border-radius: 4px;
  padding: 3px 10px;
  font-size: 10px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.sim-page {
  flex: 1;
  padding: 16px;
  overflow: hidden;
}

.sim-page-content {
  font-size: 11px;
  color: var(--text-dim);
  line-height: 2;
}

.sim-page-content .sim-heading {
  font-size: 16px;
  color: var(--text-secondary);
  font-weight: 600;
  margin-bottom: 10px;
}

.sim-page-content .sim-table {
  width: 80%;
  border-collapse: collapse;
  margin-top: 10px;
}

.sim-table th,
.sim-table td {
  border: 1px solid #222248;
  padding: 6px 10px;
  font-size: 10px;
  text-align: left;
}

.sim-table th {
  background: #15152e;
  color: var(--text-secondary);
}

.sim-table td {
  color: var(--text-dim);
}

.sim-btn-row {
  margin-top: 14px;
  display: flex;
  gap: 8px;
}

.sim-btn {
  padding: 5px 16px;
  border-radius: 4px;
  font-size: 10px;
  border: 1px solid #333;
  background: transparent;
  color: var(--text-dim);
  cursor: default;
}

.sim-btn.primary { background: #3b82f6; color: #fff; border-color: #3b82f6; }
.sim-btn.danger  { background: #dc2626; color: #fff; border-color: #dc2626; }

/* AI Crosshair */
.ai-crosshair {
  position: absolute;
  pointer-events: none;
  z-index: 10;
  transition:
    left  0.6s cubic-bezier(0.4, 0, 0.2, 1),
    top   0.6s cubic-bezier(0.4, 0, 0.2, 1);
}

.crosshair-ring {
  width: 40px;
  height: 40px;
  border: 2px solid var(--red);
  border-radius: 50%;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: crosshair-pulse 1.5s infinite;
}

.crosshair-dot {
  width: 6px;
  height: 6px;
  background: var(--red);
  border-radius: 50%;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  box-shadow: 0 0 10px var(--red);
}

.crosshair-line-h,
.crosshair-line-v {
  position: absolute;
  background: rgba(239, 68, 68, 0.4);
}

.crosshair-line-h {
  width: 20px;
  height: 1px;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

.crosshair-line-v {
  width: 1px;
  height: 20px;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}

@keyframes crosshair-pulse {
  0%, 100% { transform: translate(-50%, -50%) scale(1); opacity: 1; }
  50%       { transform: translate(-50%, -50%) scale(1.3); opacity: 0.4; }
}

/* Bounding box */
.ai-bbox {
  position: absolute;
  border: 2px solid var(--red);
  border-radius: 3px;
  pointer-events: none;
  z-index: 9;
  box-shadow: 0 0 12px rgba(239, 68, 68, 0.3);
  transition:
    left   0.5s cubic-bezier(0.4, 0, 0.2, 1),
    top    0.5s cubic-bezier(0.4, 0, 0.2, 1),
    width  0.5s cubic-bezier(0.4, 0, 0.2, 1),
    height 0.5s cubic-bezier(0.4, 0, 0.2, 1);
}

.ai-bbox::before {
  content: 'AI Focus';
  position: absolute;
  top: -18px;
  left: -2px;
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--red);
  background: rgba(239, 68, 68, 0.15);
  padding: 1px 6px;
  border-radius: 3px;
  white-space: nowrap;
}
</style>
