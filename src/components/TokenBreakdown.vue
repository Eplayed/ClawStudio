<script setup lang="ts">
/**
 * TokenBreakdown.vue - Token 分解条形图
 */
import { computed } from 'vue'

const props = defineProps<{
  inputTokens: number
  outputTokens: number
  imageTokens: number
}>()

const total = computed(() => props.inputTokens + props.outputTokens + props.imageTokens)

const inputPct = computed(() => total.value > 0 ? (props.inputTokens / total.value * 100) : 0)
const outputPct = computed(() => total.value > 0 ? (props.outputTokens / total.value * 100) : 0)
const imagePct = computed(() => total.value > 0 ? (props.imageTokens / total.value * 100) : 0)

function formatNum(n: number): string {
  return n.toLocaleString()
}
</script>

<template>
  <div class="token-breakdown">
    <h3 class="section-title">📊 Token 分解</h3>
    
    <div class="token-bars">
      <!-- Input -->
      <div class="token-row">
        <div class="token-label">
          <span class="icon">📥</span> Input Tokens
        </div>
        <div class="bar-wrap">
          <div class="bar-track">
            <div class="bar-fill cyan" :style="{ width: inputPct + '%' }">
              <span class="pct-label">{{ inputPct.toFixed(1) }}%</span>
            </div>
          </div>
          <span class="num">{{ formatNum(inputTokens) }}</span>
        </div>
      </div>

      <!-- Output -->
      <div class="token-row">
        <div class="token-label">
          <span class="icon">📤</span> Output Tokens
        </div>
        <div class="bar-wrap">
          <div class="bar-track">
            <div class="bar-fill green" :style="{ width: outputPct + '%' }">
              <span class="pct-label">{{ outputPct.toFixed(1) }}%</span>
            </div>
          </div>
          <span class="num">{{ formatNum(outputTokens) }}</span>
        </div>
      </div>

      <!-- Image -->
      <div class="token-row">
        <div class="token-label">
          <span class="icon">📷</span> Image (截图)
        </div>
        <div class="bar-wrap">
          <div class="bar-track">
            <div class="bar-fill amber" :style="{ width: imagePct + '%' }">
              <span class="pct-label">{{ imagePct.toFixed(1) }}%</span>
            </div>
          </div>
          <span class="num">{{ formatNum(imageTokens) }}</span>
        </div>
      </div>
    </div>

    <div class="token-total">
      <span>总计 Total Tokens</span>
      <span class="total-num">{{ formatNum(total) }}</span>
    </div>
  </div>
</template>

<style scoped>
.token-breakdown {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 16px;
}

.token-bars {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.token-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.token-label {
  font-size: 12px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.token-label .icon {
  font-size: 13px;
}

.bar-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bar-track {
  flex: 1;
  height: 20px;
  background: var(--border);
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.bar-fill {
  height: 100%;
  border-radius: 4px;
  display: flex;
  align-items: center;
  padding-left: 8px;
  transition: width 0.5s cubic-bezier(0.4, 0, 0.2, 1);
  min-width: 50px;
}

.bar-fill.cyan {
  background: linear-gradient(90deg, var(--cyan), var(--cyan-dim));
}
.bar-fill.green {
  background: linear-gradient(90deg, var(--green), var(--green-dim));
}
.bar-fill.amber {
  background: linear-gradient(90deg, var(--amber), var(--amber-dim));
}

.pct-label {
  font-size: 10px;
  font-family: var(--font-mono);
  color: #fff;
  font-weight: 600;
}

.num {
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-dim);
  min-width: 70px;
  text-align: right;
}

.token-total {
  margin-top: 16px;
  padding-top: 14px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
}

.total-num {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--text-primary);
}
</style>
