<script setup lang="ts">
/**
 * FuelGauge.vue - 预算油表组件
 * 圆形仪表盘显示当前费用占预算比例
 */
import { computed } from 'vue'
import { getBudgetStatus, getBudgetColor, type BudgetStatus } from '@/utils/costCalculator'

const props = defineProps<{
  current: number
  limit: number
}>()

const circumference = 2 * Math.PI * 75 // r=75
const ratio = computed(() => {
  if (props.limit <= 0) return 0
  return Math.min(1, props.current / props.limit)
})

const dashOffset = computed(() => {
  return circumference * (1 - ratio.value)
})

const status = computed<BudgetStatus>(() => getBudgetStatus(props.current, props.limit))
const color = computed(() => getBudgetColor(status.value))

const percentDisplay = computed(() => (ratio.value * 100).toFixed(1))
const currentDisplay = computed(() => props.current.toFixed(2))
const limitDisplay = computed(() => props.limit.toFixed(2))
</script>

<template>
  <div class="fuel-gauge-wrap">
    <div class="fuel-gauge">
      <svg viewBox="0 0 180 180">
        <defs>
          <linearGradient id="fuelGradGreen" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#22c55e" />
            <stop offset="100%" stop-color="#16a34a" />
          </linearGradient>
          <linearGradient id="fuelGradAmber" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#f0a030" />
            <stop offset="100%" stop-color="#d97706" />
          </linearGradient>
          <linearGradient id="fuelGradRed" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stop-color="#ef4444" />
            <stop offset="100%" stop-color="#dc2626" />
          </linearGradient>
        </defs>
        <!-- Track -->
        <circle
          cx="90" cy="90" r="75"
          fill="none"
          stroke="var(--border)"
          stroke-width="10"
        />
        <!-- Fill arc -->
        <circle
          cx="90" cy="90" r="75"
          fill="none"
          :stroke="status === 'safe' ? 'url(#fuelGradGreen)' : status === 'warning' ? 'url(#fuelGradAmber)' : 'url(#fuelGradRed)'"
          stroke-width="10"
          stroke-linecap="round"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="dashOffset"
          transform="rotate(-90 90 90)"
          class="fuel-arc"
        />
        <!-- Center text will be overlaid via HTML -->
      </svg>
      <div class="gauge-center">
        <div class="gauge-value" :style="{ color }">${{ currentDisplay }}</div>
        <div class="gauge-limit">/ ${{ limitDisplay }} 上限</div>
        <div class="gauge-percent" :style="{ background: color }">{{ percentDisplay }}%</div>
      </div>
    </div>
    <div class="gauge-legend">
      <div class="legend-item"><span class="dot safe"></span> 安全</div>
      <div class="legend-item"><span class="dot warn"></span> 警告</div>
      <div class="legend-item"><span class="dot danger"></span> 危险</div>
    </div>
  </div>
</template>

<style scoped>
.fuel-gauge-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px;
}

.fuel-gauge {
  position: relative;
  width: 180px;
  height: 180px;
}

.fuel-gauge svg {
  width: 100%;
  height: 100%;
}

.fuel-arc {
  transition: stroke-dashoffset 0.8s cubic-bezier(0.4, 0, 0.2, 1);
}

.gauge-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  text-align: center;
}

.gauge-value {
  font-size: 28px;
  font-weight: 700;
  font-family: var(--font-mono);
  transition: color 0.3s;
}

.gauge-limit {
  font-size: 11px;
  color: var(--text-dim);
  margin-top: 2px;
}

.gauge-percent {
  display: inline-block;
  margin-top: 8px;
  padding: 2px 10px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 600;
  color: #fff;
}

.gauge-legend {
  display: flex;
  gap: 16px;
  margin-top: 16px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-dim);
}

.legend-item .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.dot.safe { background: var(--green); }
.dot.warn { background: var(--amber); }
.dot.danger { background: var(--red); }
</style>
