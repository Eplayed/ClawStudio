<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits(['complete', 'next', 'back'])

const models = [
  {
    id: 'claude-3-5-sonnet-20241022',
    name: 'Claude 3.5 Sonnet',
    provider: 'Anthropic',
    price: '$3/M 输入 / $15/M 输出',
    recommended: true,
    description: '平衡性能与成本，适合大多数任务'
  },
  {
    id: 'claude-3-opus-20240229',
    name: 'Claude 3 Opus',
    provider: 'Anthropic',
    price: '$15/M 输入 / $75/M 输出',
    recommended: false,
    description: '最强大的 Claude 模型，适合复杂推理'
  },
  {
    id: 'gpt-4o',
    name: 'GPT-4o',
    provider: 'OpenAI',
    price: '$5/M 输入 / $15/M 输出',
    recommended: false,
    description: 'OpenAI 最新多模态模型'
  },
  {
    id: 'gpt-4-turbo',
    name: 'GPT-4 Turbo',
    provider: 'OpenAI',
    price: '$10/M 输入 / $30/M 输出',
    recommended: false,
    description: '快速响应，适合实时交互'
  }
]

const selectedModel = ref('')

function selectModel(modelId: string) {
  selectedModel.value = modelId
}

function proceed() {
  if (selectedModel.value) {
    emit('complete', { modelSelected: selectedModel.value })
    emit('next')
  }
}
</script>

<template>
  <div class="model-selector">
    <h2>🧠 选择模型</h2>
    <p class="subtitle">选择默认的 AI 模型</p>

    <div class="model-grid">
      <div 
        v-for="model in models" 
        :key="model.id"
        class="model-card"
        :class="{ selected: selectedModel === model.id }"
        @click="selectModel(model.id)"
      >
        <div class="card-header">
          <span class="provider-badge">{{ model.provider }}</span>
          <span v-if="model.recommended" class="recommended-badge">推荐</span>
        </div>
        
        <h3>{{ model.name }}</h3>
        <p class="description">{{ model.description }}</p>
        
        <div class="price">{{ model.price }}</div>
        
        <div class="select-indicator">
          {{ selectedModel === model.id ? '✓ 已选择' : '点击选择' }}
        </div>
      </div>
    </div>

    <div class="footer-actions">
      <button class="btn btn-secondary" @click="$emit('back')">
        ← 上一步
      </button>
      <button 
        class="btn btn-primary" 
        :disabled="!selectedModel"
        @click="proceed"
      >
        继续 →
      </button>
    </div>
  </div>
</template>

<style scoped>
.model-selector {
  max-width: 800px;
  margin: 0 auto;
}

h2 {
  font-size: 24px;
  margin-bottom: 8px;
}

.subtitle {
  color: var(--text-secondary);
  margin-bottom: 32px;
}

.model-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.model-card {
  background: var(--bg-card);
  border: 2px solid transparent;
  border-radius: 12px;
  padding: 20px;
  cursor: pointer;
  transition: all 0.2s;
}

.model-card:hover {
  border-color: var(--border-color);
}

.model-card.selected {
  border-color: var(--cyan);
  background: rgba(6, 214, 214, 0.05);
}

.card-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
}

.provider-badge {
  font-size: 12px;
  padding: 4px 8px;
  background: var(--bg-base);
  border-radius: 4px;
  color: var(--text-secondary);
}

.recommended-badge {
  font-size: 12px;
  padding: 4px 8px;
  background: var(--cyan);
  color: var(--bg-deep);
  border-radius: 4px;
  font-weight: 600;
}

.model-card h3 {
  font-size: 18px;
  margin-bottom: 8px;
}

.description {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 12px;
}

.price {
  font-size: 13px;
  color: var(--amber);
  margin-bottom: 12px;
}

.select-indicator {
  font-size: 14px;
  color: var(--text-secondary);
  text-align: center;
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
}

.model-card.selected .select-indicator {
  color: var(--cyan);
}

.footer-actions {
  display: flex;
  justify-content: space-between;
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  transition: all 0.2s;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}
</style>
