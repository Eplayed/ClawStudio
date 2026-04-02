<script setup lang="ts">
import { useProxyStore } from '@/stores/proxy'

const proxy = useProxyStore()

const handleApprove = () => {
  proxy.hitlApprove()
}

const handleReject = () => {
  const reason = prompt('请输入拒绝理由 (可选)：', 'User rejected this action')
  if (reason !== null) {
    proxy.hitlReject(reason)
  }
}
</script>

<template>
  <div v-if="proxy.hitlPending" class="hitl-bar">
    <div class="hitl-content">
      <span class="warning-icon">⚠️</span>
      <div class="hitl-details">
        <strong>拦截到高危操作：</strong>
        <span class="tool-name">{{ proxy.hitlPending.tool }}</span>
        <pre class="params">{{ JSON.stringify(proxy.hitlPending.params, null, 2) }}</pre>
      </div>
    </div>
    <div class="hitl-actions">
      <button class="btn btn-success" @click="handleApprove">允许执行</button>
      <button class="btn btn-error" @click="handleReject">拒绝执行</button>
    </div>
  </div>
</template>

<style scoped>
.hitl-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--color-warning-bg, #fff3cd);
  border: 1px solid var(--color-warning-border, #ffe69c);
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 1rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  position: relative;
  z-index: 100;
}

.hitl-content {
  display: flex;
  align-items: flex-start;
  gap: 1rem;
}

.warning-icon {
  font-size: 1.5rem;
}

.hitl-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tool-name {
  font-family: monospace;
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.params {
  font-size: 0.85rem;
  background: rgba(0, 0, 0, 0.05);
  padding: 0.5rem;
  border-radius: 4px;
  max-height: 100px;
  overflow-y: auto;
  margin: 0;
}

.hitl-actions {
  display: flex;
  gap: 1rem;
}

.btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.btn-success {
  background: var(--color-success, #28a745);
  color: white;
}

.btn-error {
  background: var(--color-error, #dc3545);
  color: white;
}
</style>