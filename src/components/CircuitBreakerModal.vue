<script setup lang="ts">
import { computed } from 'vue'
import { useProxyStore } from '@/stores/proxy'

const proxyStore = useProxyStore()

const showModal = computed(() => proxyStore.circuitBroken)

async function resetAndResume() {
  try {
    await proxyStore.resetCost()
  } catch (e) {
    console.error('Reset failed:', e)
  }
}

async function increaseBudget() {
  const newLimit = proxyStore.budgetLimit + 50
  try {
    await proxyStore.setBudgetLimit(newLimit)
    await resetAndResume()
  } catch (e) {
    console.error('Set budget failed:', e)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="showModal" class="modal-overlay" @click.self="/* do nothing on backdrop */">
      <div class="modal-box circuit-modal">
        <div class="warning-icon">⚠️</div>
        <h2>预算已超支</h2>
        <p class="status-text">代理已被暂停</p>
        
        <div class="cost-info">
          <div class="cost-row">
            <span class="label">当前费用:</span>
            <span class="value danger">${{ proxyStore.totalCost.toFixed(4) }}</span>
          </div>
          <div class="cost-row">
            <span class="label">预算上限:</span>
            <span class="value">${{ proxyStore.budgetLimit.toFixed(2) }}</span>
          </div>
        </div>

        <div class="actions">
          <button class="btn btn-primary" @click="resetAndResume">
            🔄 重置计数并继续
          </button>
          <button class="btn btn-secondary" @click="increaseBudget">
            💰 增加预算 +$50
          </button>
        </div>

        <p class="hint">重置后当前会话的费用将从零开始计算</p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(4px);
}

.modal-box {
  background: var(--bg-deep, #1a1a2e);
  border: 1px solid var(--border, #333);
  border-radius: 12px;
  padding: 32px;
  max-width: 420px;
  width: 90%;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.circuit-modal {
  border-color: #f59e0b;
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.1) 0%, var(--bg-deep, #1a1a2e) 100%);
}

.warning-icon {
  font-size: 48px;
  margin-bottom: 8px;
}

h2 {
  color: #f59e0b;
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.status-text {
  color: var(--text-muted, #888);
  margin: 0 0 24px 0;
}

.cost-info {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 24px;
}

.cost-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
}

.cost-row:not(:last-child) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.label {
  color: var(--text-muted, #888);
}

.value {
  color: var(--text, #fff);
  font-weight: 500;
}

.value.danger {
  color: #ef4444;
  font-weight: 600;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  border: none;
  font-size: 0.95rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #f59e0b;
  color: #000;
}

.btn-primary:hover {
  background: #d97706;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text, #fff);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
}

.hint {
  margin-top: 16px;
  font-size: 0.85rem;
  color: var(--text-muted, #666);
}
</style>
