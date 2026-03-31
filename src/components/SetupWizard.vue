<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import EnvDetector from './EnvDetector.vue'
import NodeInstaller from './NodeInstaller.vue'
import ClawInstaller from './ClawInstaller.vue'
import ApiKeySetup from './ApiKeySetup.vue'
import ModelSelector from './ModelSelector.vue'
import ChannelSetup from './ChannelSetup.vue'
import LaunchGateway from './LaunchGateway.vue'

const currentStep = ref(1)
const totalSteps = 7

const steps = [
  { num: 1, name: '环境检测', component: EnvDetector },
  { num: 2, name: '安装 Node.js', component: NodeInstaller },
  { num: 3, name: '安装 OpenClaw', component: ClawInstaller },
  { num: 4, name: '配置 API Key', component: ApiKeySetup },
  { num: 5, name: '选择模型', component: ModelSelector },
  { num: 6, name: '配置通道', component: ChannelSetup },
  { num: 7, name: '启动 Gateway', component: LaunchGateway },
]

const currentComponent = computed(() => steps[currentStep.value - 1].component)

const canGoBack = computed(() => currentStep.value > 1)
const canGoNext = computed(() => currentStep.value < totalSteps && isStepComplete.value)
const isStepComplete = ref(false)

// Setup configuration state
const setupConfig = ref({
  nodeInstalled: false,
  openclawInstalled: false,
  apiKeyConfigured: false,
  modelSelected: '',
  channelsConfigured: false,
  gatewayStarted: false,
})

function nextStep() {
  if (canGoNext.value && currentStep.value < totalSteps) {
    currentStep.value++
    isStepComplete.value = false
  }
}

function prevStep() {
  if (canGoBack.value) {
    currentStep.value--
    isStepComplete.value = false
  }
}

function onStepComplete(data: any) {
  isStepComplete.value = true
  if (data) {
    Object.assign(setupConfig.value, data)
  }
}

// Listen for environment detection results
onMounted(async () => {
  try {
    const envStatus = await invoke('get_env_status')
    
    // Auto-determine starting step
    const node = (envStatus as any).node
    const openclaw = (envStatus as any).openclaw
    const gateway = (envStatus as any).gateway
    
    if (node?.installed && openclaw?.installed && gateway?.running) {
      // Already fully setup, skip to dashboard
      currentStep.value = 7
    } else if (node?.installed && openclaw?.installed) {
      currentStep.value = 4 // API Key config
    } else if (node?.installed) {
      currentStep.value = 3 // OpenClaw install
    } else {
      currentStep.value = 1 // Start from environment detection
    }
  } catch (e) {
    console.error('Failed to check environment:', e)
    currentStep.value = 1
  }
})
</script>

<template>
  <div class="setup-wizard">
    <!-- Progress Header -->
    <div class="wizard-header">
      <div class="progress-bar">
        <div 
          class="progress-fill" 
          :style="{ width: `${(currentStep / totalSteps) * 100}%` }"
        ></div>
      </div>
      
      <div class="steps-indicator">
        <div 
          v-for="step in steps" 
          :key="step.num"
          class="step-dot"
          :class="{ 
            active: step.num === currentStep,
            completed: step.num < currentStep,
            pending: step.num > currentStep
          }"
          @click="step.num < currentStep && (currentStep = step.num)"
        >
          <span class="step-num">{{ step.num < currentStep ? '✓' : step.num }}</span>
          <span class="step-name">{{ step.name }}</span>
        </div>
      </div>
    </div>

    <!-- Step Content -->
    <div class="wizard-content">
      <Transition name="fade" mode="out-in">
        <component 
          :is="currentComponent"
          :key="currentStep"
          @complete="onStepComplete"
          @next="nextStep"
          @back="prevStep"
        />
      </Transition>
    </div>

    <!-- Navigation Footer -->
    <div class="wizard-footer">
      <button 
        class="btn btn-secondary"
        :disabled="!canGoBack"
        @click="prevStep"
      >
        ← 上一步
      </button>
      
      <button 
        v-if="currentStep < totalSteps"
        class="btn btn-primary"
        :disabled="!canGoNext"
        @click="nextStep"
      >
        下一步 →
      </button>
      
      <button 
        v-else
        class="btn btn-success"
        @click="$emit('complete')"
      >
        🚀 进入控制台
      </button>
    </div>
  </div>
</template>

<style scoped>
.setup-wizard {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-deep);
  color: var(--text-primary);
}

.wizard-header {
  padding: 24px;
  background: var(--bg-base);
  border-bottom: 1px solid var(--border-color);
}

.progress-bar {
  height: 4px;
  background: var(--bg-card);
  border-radius: 2px;
  overflow: hidden;
  margin-bottom: 24px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--cyan), var(--amber));
  transition: width 0.3s ease;
}

.steps-indicator {
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.step-dot {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.step-dot.pending {
  opacity: 0.4;
  cursor: not-allowed;
}

.step-dot.completed {
  opacity: 0.7;
}

.step-dot.active {
  opacity: 1;
}

.step-num {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--bg-card);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  font-size: 14px;
}

.step-dot.active .step-num {
  background: var(--cyan);
  color: var(--bg-deep);
}

.step-dot.completed .step-num {
  background: var(--green);
  color: white;
}

.step-name {
  font-size: 12px;
  color: var(--text-secondary);
}

.step-dot.active .step-name {
  color: var(--cyan);
}

.wizard-content {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
}

.wizard-footer {
  display: flex;
  justify-content: space-between;
  padding: 24px;
  background: var(--bg-base);
  border-top: 1px solid var(--border-color);
}

.btn {
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--cyan);
  color: var(--bg-deep);
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-secondary {
  background: var(--bg-card);
  color: var(--text-primary);
}

.btn-success {
  background: var(--green);
  color: white;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
