<script setup lang="ts">
/**
 * TracePlayer.vue - 历史回放播放器
 * 截图 + 日志同步 + 进度条 + 速度控制
 */
import { ref, computed, onUnmounted } from 'vue'

export interface TraceStep {
  label: string
  body: string  // HTML content of the simulated screen at this step
  timestamp: string
  action: string
}

const props = defineProps<{
  steps: TraceStep[]
}>()

const emit = defineEmits<{
  close: []
}>()

// Playback state
const currentStep = ref(0)
const isPlaying = ref(false)
const playbackSpeed = ref(1) // 0.5, 1, 2
let playInterval: ReturnType<typeof setInterval> | undefined

const SPEED_MS: Record<number, number> = {
  0.5: 3000,
  1:   1500,
  2:   750,
}

const totalSteps = computed(() => props.steps.length)
const progress = computed(() =>
  totalSteps.value > 1 ? (currentStep.value / (totalSteps.value - 1)) * 100 : 0
)

const currentData = computed(() => props.steps[currentStep.value] || props.steps[0])

const elapsedTime = computed(() => {
  const ms = currentStep.value * SPEED_MS[playbackSpeed.value]
  const s = Math.floor(ms / 1000)
  const m = Math.floor(s / 60)
  return `${m}:${String(s % 60).padStart(2, '0')}`
})

const totalTime = computed(() => {
  const ms = (totalSteps.value - 1) * SPEED_MS[playbackSpeed.value]
  const s = Math.floor(ms / 1000)
  const m = Math.floor(s / 60)
  return `${m}:${String(s % 60).padStart(2, '0')}`
})

function goToStep(idx: number) {
  currentStep.value = Math.max(0, Math.min(idx, totalSteps.value - 1))
  if (isPlaying.value) {
    stopPlay()
  }
}

function togglePlay() {
  if (isPlaying.value) {
    stopPlay()
  } else {
    startPlay()
  }
}

function startPlay() {
  if (totalSteps.value <= 1) return
  isPlaying.value = true
  playInterval = setInterval(() => {
    if (currentStep.value < totalSteps.value - 1) {
      currentStep.value++
    } else {
      stopPlay()
    }
  }, SPEED_MS[playbackSpeed.value])
}

function stopPlay() {
  isPlaying.value = false
  if (playInterval) {
    clearInterval(playInterval)
    playInterval = undefined
  }
}

function prevStep() {
  goToStep(currentStep.value - 1)
}

function nextStep() {
  goToStep(currentStep.value + 1)
}

function setSpeed(s: number) {
  playbackSpeed.value = s
  if (isPlaying.value) {
    stopPlay()
    startPlay()
  }
}

function onScrubberClick(e: MouseEvent) {
  const bar = e.currentTarget as HTMLElement
  const rect = bar.getBoundingClientRect()
  const ratio = (e.clientX - rect.left) / rect.width
  const idx = Math.round(ratio * (totalSteps.value - 1))
  goToStep(idx)
}

function exportTrace() {
  const content = [
    '=== ClawStudio Trace Export ===',
    `Total steps: ${totalSteps.value}`,
    `Duration: ${elapsedTime.value} / ${totalTime.value}`,
    `Speed: ${playbackSpeed.value}x`,
    '',
    ...props.steps.map((s, i) =>
      `[Step ${i + 1}] ${s.timestamp} — ${s.action}\n${s.body}`
    ),
  ].join('\n\n')

  const blob = new Blob([content], { type: 'text/plain' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `clawstudio-trace-${Date.now()}.txt`
  a.click()
  URL.revokeObjectURL(url)
}

onUnmounted(stopPlay)
</script>

<template>
  <div class="trace-player">
    <!-- Screen + Log side by side -->
    <div class="trace-player-view">
      <!-- Screen (left) -->
      <div class="trace-screen">
        <div class="trace-screen-img">
          <div class="trace-sim-bar">
            <div class="tdot r"></div>
            <div class="tdot y"></div>
            <div class="tdot g"></div>
          </div>
          <div class="trace-sim-body" v-html="currentData.body"></div>
          <div class="trace-step-label">
            Step {{ currentStep + 1 }}/{{ totalSteps }} — {{ currentData.label }}
          </div>
        </div>
      </div>

      <!-- Log panel (right) -->
      <div class="trace-log-panel">
        <div class="trace-log-header">
          📋 操作日志
        </div>
        <div class="trace-log-list">
          <div
            v-for="(step, idx) in steps"
            :key="idx"
            class="trace-log-item"
            :class="{ active: idx === currentStep }"
            @click="goToStep(idx)"
          >
            <span class="tl-time">{{ step.timestamp }}</span>
            <span class="tl-text">
              <span class="tl-badge" :style="{ color: idx <= currentStep ? 'var(--cyan)' : 'var(--text-dim)' }">
                {{ step.action }}
              </span>
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Controls bar -->
    <div class="trace-controls">
      <!-- Prev -->
      <button class="tc-btn" @click="prevStep" title="上一步">◀</button>

      <!-- Play/Pause -->
      <button
        class="tc-btn"
        :class="{ active: isPlaying }"
        @click="togglePlay"
        :title="isPlaying ? '暂停' : '播放'"
      >
        {{ isPlaying ? '⏸' : '▶' }}
      </button>

      <!-- Next -->
      <button class="tc-btn" @click="nextStep" title="下一步">▶</button>

      <!-- Scrubber -->
      <div class="trace-scrubber">
        <div class="scrubber-bar" @click="onScrubberClick">
          <div class="scrubber-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="scrubber-ticks">
          <span>Step 1</span>
          <span>Step {{ totalSteps }}</span>
        </div>
      </div>

      <!-- Time -->
      <div class="tc-time">{{ elapsedTime }} / {{ totalTime }}</div>

      <!-- Speed -->
      <select class="tc-speed-select" v-model.number="playbackSpeed" @change="setSpeed(playbackSpeed)">
        <option :value="0.5">0.5×</option>
        <option :value="1">1×</option>
        <option :value="2">2×</option>
      </select>

      <!-- Export -->
      <button class="tc-export" @click="exportTrace">
        📥 导出
      </button>
    </div>
  </div>
</template>

<style scoped>
.trace-player {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  margin-bottom: 20px;
}

.trace-player-view {
  display: flex;
  height: 400px;
}

.trace-screen {
  flex: 6;
  background: #0d0d20;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.trace-screen-img {
  width: 90%;
  height: 90%;
  border-radius: var(--radius-sm);
  background: linear-gradient(135deg, #1a1a3a, #12122a);
  border: 1px solid #222248;
  display: flex;
  flex-direction: column;
  position: relative;
  overflow: hidden;
}

.trace-sim-bar {
  height: 28px;
  background: #0d0d1e;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 6px;
  border-bottom: 1px solid #222248;
  flex-shrink: 0;
}

.trace-sim-bar .tdot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
}

.trace-sim-bar .tdot.r { background: #ef4444; }
.trace-sim-bar .tdot.y { background: #eab308; }
.trace-sim-bar .tdot.g { background: #22c55e; }

.trace-sim-body {
  flex: 1;
  padding: 16px;
  font-size: 11px;
  color: var(--text-dim);
  line-height: 2.2;
  overflow: hidden;
}

.trace-step-label {
  position: absolute;
  bottom: 10px;
  left: 10px;
  background: rgba(0, 0, 0, 0.7);
  color: var(--cyan);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--cyan-dim);
  white-space: nowrap;
}

/* Log panel */
.trace-log-panel {
  flex: 4;
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
}

.trace-log-header {
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  font-size: 12px;
  font-weight: 600;
  flex-shrink: 0;
}

.trace-log-list {
  flex: 1;
  overflow-y: auto;
  padding: 6px 0;
  font-family: var(--font-mono);
  font-size: 11px;
}

.trace-log-item {
  padding: 5px 14px;
  display: flex;
  gap: 8px;
  border-left: 2px solid transparent;
  cursor: pointer;
  transition: background 0.15s;
  align-items: center;
}

.trace-log-item:hover {
  background: rgba(255, 255, 255, 0.02);
}

.trace-log-item.active {
  background: var(--cyan-glow);
  border-left-color: var(--cyan);
}

.tl-time {
  color: var(--text-dim);
  font-size: 10px;
  min-width: 48px;
}

.tl-badge {
  font-size: 11px;
}

/* Controls */
.trace-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 18px;
  border-top: 1px solid var(--border);
  background: var(--bg-base);
}

.tc-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
  flex-shrink: 0;
}

.tc-btn:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.tc-btn.active {
  background: var(--cyan-glow);
  border-color: var(--cyan-dim);
  color: var(--cyan);
}

.trace-scrubber {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.scrubber-bar {
  width: 100%;
  height: 6px;
  background: var(--border);
  border-radius: 3px;
  position: relative;
  cursor: pointer;
  overflow: visible;
}

.scrubber-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--cyan), var(--green));
  border-radius: 3px;
  transition: width 0.3s;
  position: relative;
}

.scrubber-fill::after {
  content: '';
  position: absolute;
  right: -5px;
  top: -2px;
  width: 10px;
  height: 10px;
  background: #fff;
  border-radius: 50%;
  box-shadow: 0 0 8px var(--cyan-glow-strong);
}

.scrubber-ticks {
  display: flex;
  justify-content: space-between;
  font-size: 9px;
  font-family: var(--font-mono);
  color: var(--text-dim);
}

.tc-time {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-secondary);
  min-width: 80px;
  text-align: center;
  flex-shrink: 0;
}

.tc-speed-select {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 4px 8px;
  cursor: pointer;
  flex-shrink: 0;
  outline: none;
}

.tc-export {
  padding: 6px 14px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-card);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  font-family: var(--font-ui);
  transition: all 0.2s;
  flex-shrink: 0;
  white-space: nowrap;
}

.tc-export:hover {
  border-color: var(--cyan-dim);
  color: var(--cyan);
}
</style>
