<script setup lang="ts">
import { ref, computed } from 'vue'
import { useProxyStore } from '@/stores/proxy'

const proxy = useProxyStore()
const rejectReason = ref('')
const showRejectForm = ref(false)

const hasPending = computed(() => proxy.hitlPending !== null)

function handleApprove() {
  proxy.hitlApprove()
}

function handleRejectClick() {
  showRejectForm.value = true
}

function confirmReject() {
  proxy.hitlReject(rejectReason.value || undefined)
  showRejectForm.value = false
  rejectReason.value = ''
}

function cancelReject() {
  showRejectForm.value = false
  rejectReason.value = ''
}

function formatParams(params: any): string {
  try {
    return JSON.stringify(params, null, 2)
  } catch {
    return String(params)
  }
}
</script>

<template>
  <!-- Inline bar for small notifications -->
  <div v-if="hasPending && !showRejectForm" class="hitl-bar">
    <div class="hitl-content">
      <span class="warning-icon">⚠️</span>
      <div class="hitl-details">
        <strong>拦截到高危操作：</strong>
        <code class="tool-name">{{ proxy.hitlPending?.tool }}</code>
      </div>
    </div>
    <div class="hitl-actions">
      <button class="btn btn-success" @click="handleApprove">✅ 允许</button>
      <button class="btn btn-error" @click="handleRejectClick">🚫 拒绝</button>
    </div>
  </div>

  <!-- Reject reason modal -->
  <Teleport to="body">
    <div v-if="showRejectForm" class="modal-overlay" @click.self="cancelReject">
      <div class="modal-box hitl-modal">
        <div class="modal-header">
          <span class="warning-icon">🚫</span>
          <h2>拒绝操作</h2>
        </div>

        <div class="request-info">
          <div class="info-row">
            <span class="label">工具:</span>
            <code class="tool-badge">{{ proxy.hitlPending?.tool }}</code>
          </div>
          <div class="info-row">
            <span class="label">请求ID:</span>
            <code class="id-badge">{{ proxy.hitlPending?.request_id?.slice(0, 8) }}...</code>
          </div>
        </div>

        <div class="params-section">
          <p class="section-title">参数详情:</p>
          <pre class="params-display">{{ formatParams(proxy.hitlPending?.params) }}</pre>
        </div>

        <div class="reason-section">
          <label for="reject-reason" class="section-title">拒绝理由 (可选):</label>
          <textarea
            id="reject-reason"
            v-model="rejectReason"
            class="reason-input"
            placeholder="输入拒绝理由，AI 将看到这个提示..."
            rows="3"
          ></textarea>
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="cancelReject">取消</button>
          <button class="btn btn-danger" @click="confirmReject">确认拒绝</button>
        </div>

        <p class="hint">拒绝后，AI 将收到错误提示并跳过此操作</p>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.hitl-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: linear-gradient(135deg, #fff3cd 0%, #ffeeba 100%);
  border: 1px solid #ffc107;
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 12px;
  box-shadow: 0 2px 8px rgba(255, 193, 7, 0.3);
}

.hitl-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

.warning-icon {
  font-size: 1.5rem;
}

.hitl-details {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.tool-name {
  font-family: monospace;
  font-size: 0.85rem;
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
}

.hitl-actions {
  display: flex;
  gap: 8px;
}

.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.btn-success {
  background: #28a745;
  color: white;
}

.btn-success:hover {
  background: #218838;
}

.btn-error {
  background: #dc3545;
  color: white;
}

.btn-error:hover {
  background: #c82333;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9998;
  backdrop-filter: blur(4px);
}

.modal-box {
  background: var(--bg-deep, #1a1a2e);
  border: 1px solid var(--border, #333);
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.hitl-modal {
  border-color: #dc3545;
  background: linear-gradient(135deg, rgba(220, 53, 69, 0.1) 0%, var(--bg-deep, #1a1a2e) 100%);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.modal-header h2 {
  margin: 0;
  color: #dc3545;
  font-size: 1.3rem;
}

.request-info {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.label {
  color: var(--text-muted, #888);
  font-size: 0.9rem;
}

.tool-badge {
  background: rgba(220, 53, 69, 0.2);
  color: #ff6b6b;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.9rem;
}

.id-badge {
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.85rem;
  color: var(--text-muted, #888);
}

.params-section {
  margin-bottom: 16px;
}

.section-title {
  margin: 0 0 8px 0;
  font-size: 0.9rem;
  color: var(--text-muted, #888);
}

.params-display {
  background: rgba(0, 0, 0, 0.4);
  border-radius: 6px;
  padding: 12px;
  font-size: 0.85rem;
  max-height: 150px;
  overflow-y: auto;
  margin: 0;
  color: var(--text, #e0e0e0);
  white-space: pre-wrap;
  word-break: break-all;
}

.reason-section {
  margin-bottom: 20px;
}

.reason-input {
  width: 100%;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border, #444);
  border-radius: 6px;
  padding: 10px;
  color: var(--text, #fff);
  font-size: 0.9rem;
  resize: vertical;
  font-family: inherit;
}

.reason-input:focus {
  outline: none;
  border-color: #dc3545;
}

.reason-input::placeholder {
  color: var(--text-muted, #666);
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text, #fff);
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.2);
}

.btn-danger {
  background: #dc3545;
  color: white;
}

.btn-danger:hover {
  background: #c82333;
}

.hint {
  margin-top: 12px;
  font-size: 0.8rem;
  color: var(--text-muted, #666);
  text-align: center;
}
</style>
