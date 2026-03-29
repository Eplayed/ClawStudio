<script setup lang="ts">
/**
 * Traces.vue - 历史回放页面
 */
import { ref, computed } from 'vue'
import TracePlayer from '@/components/TracePlayer.vue'
import TraceHistory from '@/components/TraceHistory.vue'
import type { TraceStep } from '@/components/TracePlayer.vue'

interface TraceRecord {
  id: string
  agent_id: string
  task_name: string
  status: string
  steps: number
  cost: number
  duration_sec: number
  created_at: string
}

// Demo steps for playback
const demoSteps: TraceStep[] = [
  {
    label: 'Launch',
    timestamp: '14:00:00',
    action: 'launch',
    body: `<div style="font-size:14px;color:#e8ecf4;font-weight:600;margin-bottom:10px">🚀 Agent 启动</div>
<div style="font-size:11px;color:#8892a8">正在初始化浏览器环境...</div>`,
  },
  {
    label: 'Navigate',
    timestamp: '14:00:12',
    action: 'navigate',
    body: `<div style="font-size:14px;color:#e8ecf4;font-weight:600;margin-bottom:10px">🌐 打开发票页面</div>
<div style="font-size:11px;color:#8892a8">https://invoice.example.com/monthly</div>`,
  },
  {
    label: 'Screenshot',
    timestamp: '14:00:18',
    action: 'screenshot',
    body: `<div style="font-size:14px;color:#e8ecf4;font-weight:600;margin-bottom:10px">📸 截图存档</div>
<div style="font-size:11px;color:#22c55e;margin-top:8px">✓ 截图已保存</div>`,
  },
  {
    label: 'Click Row',
    timestamp: '14:00:25',
    action: 'click',
    body: `<div style="font-size:14px;color:#e8ecf4;font-weight:600;margin-bottom:10px">🖱 点击第2行</div>
<div style="font-size:11px;color:#06d6d6;background:rgba(6,214,214,0.1);padding:4px 8px;border-radius:4px;display:inline-block">
mouse_move(x:342, y:215) → left_click()
</div>`,
  },
  {
    label: 'Extract Data',
    timestamp: '14:00:31',
    action: 'extract',
    body: `<div style="font-size:14px;color:#e8ecf4;font-weight:600;margin-bottom:10px">📋 提取数据</div>
<div style="font-size:11px;color:#22c55e">✓ INV-2024-002: ¥1,280</div>
<div style="font-size:11px;color:#22c55e;margin-top:4px">✓ 写入电子表格</div>`,
  },
  {
    label: 'Complete',
    timestamp: '14:00:42',
    action: 'complete',
    body: `<div style="font-size:14px;color:#22c55e;font-weight:600;margin-bottom:10px">✅ 任务完成</div>
<div style="font-size:11px;color:#8892a8">处理 8 张发票，耗时 42 秒</div>
<div style="font-size:11px;color:#f0a030;margin-top:8px">消耗: $0.34</div>`,
  },
]

const selectedTrace = ref<TraceRecord | null>(null)
const isPlayerVisible = ref(false)

function handleSelect(trace: TraceRecord) {
  selectedTrace.value = trace
  isPlayerVisible.value = true
}

function closePlayer() {
  isPlayerVisible.value = false
}
</script>

<template>
  <div class="page">
    <!-- Top bar -->
    <header class="topbar">
      <div class="page-title">⏱ 历史回放 Traces</div>
    </header>

    <!-- Content -->
    <div class="traces-content">
      <!-- Replay player (shown when a trace is selected) -->
      <div v-if="isPlayerVisible" class="player-section">
        <div class="player-header">
          <h2>▶ 回放: {{ selectedTrace?.task_name }}</h2>
          <button class="btn-close" @click="closePlayer">✕ 关闭</button>
        </div>
        <TracePlayer :steps="demoSteps" @close="closePlayer" />
      </div>

      <!-- History table -->
      <TraceHistory @select="handleSelect" />
    </div>
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
  padding: 0 24px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-base);
  flex-shrink: 0;
}

.page-title {
  font-size: 15px;
  font-weight: 600;
}

.traces-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px;
}

.player-section {
  margin-bottom: 20px;
}

.player-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 14px;
}

.player-header h2 {
  font-size: 14px;
  font-weight: 600;
}

.btn-close {
  padding: 6px 16px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border);
  background: var(--bg-elevated);
  color: var(--text-secondary);
  font-size: 11px;
  font-family: var(--font-ui);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-close:hover {
  border-color: var(--red-dim);
  color: var(--red);
  background: var(--red-glow);
}
</style>
