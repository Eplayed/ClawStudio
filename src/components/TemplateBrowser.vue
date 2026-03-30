<template>
  <div class="template-browser">
    <div class="browser-header">
      <h2>🎯 Agent Templates</h2>
      <div class="header-actions">
        <button @click="showImport = true" class="btn-import">
          📥 Import Template
        </button>
        <button @click="showCreate = true" class="btn-create">
          ➕ Create Template
        </button>
      </div>
    </div>

    <!-- Template Grid -->
    <div class="template-grid">
      <div 
        v-for="template in templates" 
        :key="template.id"
        class="template-card"
        @click="selectTemplate(template)"
      >
        <div class="template-header">
          <span class="template-name">{{ template.name }}</span>
          <span v-if="template.verified" class="verified-badge" title="Verified by ClawStudio">
            ✅
          </span>
        </div>
        <div class="template-author">by {{ template.author }}</div>
        <div class="template-desc">{{ template.description }}</div>
        <div class="template-tags">
          <span v-for="tag in template.tags.slice(0, 3)" :key="tag" class="tag">
            {{ tag }}
          </span>
        </div>
        <div class="template-stats">
          <span>⭐ {{ template.rating.toFixed(1) }}</span>
          <span>📥 {{ template.downloads }}</span>
        </div>
      </div>
    </div>

    <!-- Import Modal -->
    <div v-if="showImport" class="modal-overlay" @click.self="showImport = false">
      <div class="modal-content import-modal">
        <div class="modal-header">
          <h3>📥 Import Template</h3>
          <button @click="showImport = false" class="btn-close">×</button>
        </div>
        
        <div class="modal-body">
          <!-- Drop Zone -->
          <div 
            class="drop-zone"
            :class="{ dragging: isDragging }"
            @dragover.prevent="isDragging = true"
            @dragleave="isDragging = false"
            @drop.prevent="handleDrop"
          >
            <div class="drop-icon">📁</div>
            <div class="drop-text">
              Drag & drop a <code>.claw-template</code> file here
            </div>
            <div class="drop-or">or</div>
            <button @click="selectFile" class="btn-browse">
              Browse Files
            </button>
          </div>

          <!-- Deep Link Import -->
          <div class="link-import">
            <div class="divider">
              <span>Or paste a share link</span>
            </div>
            <div class="link-input">
              <input 
                v-model="importLink" 
                type="text" 
                placeholder="claw://template/..."
                class="input-field"
              />
              <button @click="importFromLink" class="btn-import-link" :disabled="!importLink">
                Import
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Template Detail Modal -->
    <div v-if="selectedTemplate" class="modal-overlay" @click.self="selectedTemplate = null">
      <div class="modal-content detail-modal">
        <div class="modal-header">
          <div>
            <h3>{{ selectedTemplate.name }}</h3>
            <div class="detail-meta">
              <span>by {{ selectedTemplate.author }}</span>
              <span>v{{ selectedTemplate.version }}</span>
              <span v-if="selectedTemplate.verified" class="verified">✅ Verified</span>
            </div>
          </div>
          <button @click="selectedTemplate = null" class="btn-close">×</button>
        </div>
        
        <div class="modal-body">
          <div class="detail-section">
            <h4>Description</h4>
            <p>{{ selectedTemplate.description }}</p>
          </div>

          <div class="detail-section">
            <h4>Configuration</h4>
            <div class="config-grid">
              <div class="config-item">
                <span class="config-label">Model</span>
                <span class="config-value">{{ selectedTemplate.model }}</span>
              </div>
              <div class="config-item">
                <span class="config-label">HITL Level</span>
                <span class="config-value">{{ selectedTemplate.hitl_level }}</span>
              </div>
              <div class="config-item">
                <span class="config-label">Computer Use</span>
                <span class="config-value">{{ selectedTemplate.computer_use ? 'Enabled' : 'Disabled' }}</span>
              </div>
              <div class="config-item">
                <span class="config-label">Max Tokens</span>
                <span class="config-value">{{ selectedTemplate.max_tokens || 'Default' }}</span>
              </div>
            </div>
          </div>

          <div class="detail-section">
            <h4>System Prompt</h4>
            <pre class="prompt-preview">{{ selectedTemplate.system_prompt }}</pre>
          </div>

          <div class="detail-section">
            <h4>Tags</h4>
            <div class="tag-list">
              <span v-for="tag in selectedTemplate.tags" :key="tag" class="tag">
                {{ tag }}
              </span>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button @click="downloadSelected" class="btn-download">
            📥 Download
          </button>
          <button @click="copyLink" class="btn-copy">
            📋 Copy Link
          </button>
          <button @click="useTemplate" class="btn-use">
            🚀 Use This Template
          </button>
        </div>
      </div>
    </div>

    <!-- Create Template Modal -->
    <div v-if="showCreate" class="modal-overlay" @click.self="showCreate = false">
      <div class="modal-content create-modal">
        <div class="modal-header">
          <h3>➕ Create Template</h3>
          <button @click="showCreate = false" class="btn-close">×</button>
        </div>
        
        <div class="modal-body">
          <div class="form-group">
            <label>Template Name</label>
            <input v-model="newTemplate.name" type="text" class="input-field" placeholder="My Agent Template" />
          </div>

          <div class="form-group">
            <label>Description</label>
            <textarea v-model="newTemplate.description" class="input-field" placeholder="Describe what this agent does..." rows="2"></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label>Model</label>
              <select v-model="newTemplate.model" class="input-field">
                <option value="claude-3-5-sonnet-20241022">Claude 3.5 Sonnet</option>
                <option value="gpt-4o">GPT-4o</option>
                <option value="deepseek-chat">DeepSeek Chat</option>
              </select>
            </div>
            <div class="form-group">
              <label>HITL Level</label>
              <select v-model="newTemplate.hitl_level" class="input-field">
                <option value="browse">Browse (strictest)</option>
                <option value="standard">Standard</option>
                <option value="auto">Auto (most permissive)</option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label>System Prompt</label>
            <textarea v-model="newTemplate.system_prompt" class="input-field prompt-input" placeholder="You are a helpful AI assistant..." rows="4"></textarea>
          </div>

          <div class="form-row">
            <div class="form-group checkbox">
              <label>
                <input type="checkbox" v-model="newTemplate.computer_use" />
                Enable Computer Use
              </label>
            </div>
          </div>

          <div v-if="newTemplate.computer_use" class="form-group">
            <label>Sandbox Image</label>
            <input v-model="newTemplate.sandbox_image" type="text" class="input-field" placeholder="dorowu/ubuntu-desktop-lxde-vnc:focal" />
          </div>

          <div class="form-group">
            <label>Tags (comma separated)</label>
            <input v-model="tagsInput" type="text" class="input-field" placeholder="automation, finance, productivity" />
          </div>
        </div>

        <div class="modal-footer">
          <button @click="showCreate = false" class="btn-cancel">Cancel</button>
          <button @click="createTemplate" class="btn-create-final" :disabled="!newTemplate.name || !newTemplate.system_prompt">
            Create Template
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { 
  TemplateManager, 
  type AgentTemplate, 
  type TemplateMeta,
  downloadTemplate,
  copyShareLink,
  parseDroppedFile 
} from '@/utils/template'

const templates = ref<TemplateMeta[]>([])
const showImport = ref(false)
const showCreate = ref(false)
const isDragging = ref(false)
const importLink = ref('')
const selectedTemplate = ref<AgentTemplate | null>(null)

const newTemplate = ref({
  name: '',
  description: '',
  model: 'claude-3-5-sonnet-20241022',
  system_prompt: '',
  computer_use: false,
  sandbox_image: '',
  hitl_level: 'standard' as 'browse' | 'standard' | 'auto',
  tags: [] as string[],
  channels: [] as string[],
})

const tagsInput = computed({
  get: () => newTemplate.value.tags.join(', '),
  set: (val: string) => {
    newTemplate.value.tags = val.split(',').map(t => t.trim()).filter(Boolean)
  }
})

onMounted(async () => {
  await loadTemplates()
})

async function loadTemplates() {
  try {
    templates.value = await TemplateManager.getBuiltinTemplates()
  } catch (error) {
    console.error('Failed to load templates:', error)
  }
}

async function selectTemplate(meta: TemplateMeta) {
  try {
    selectedTemplate.value = await TemplateManager.getBuiltinTemplate(meta.name)
  } catch (error) {
    console.error('Failed to load template:', error)
  }
}

async function handleDrop(event: DragEvent) {
  isDragging.value = false
  
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return
  
  try {
    const template = await parseDroppedFile(files[0])
    selectedTemplate.value = template
    showImport.value = false
  } catch (error) {
    alert(`Failed to import template: ${error}`)
  }
}

async function selectFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Claw Template', extensions: ['claw-template', 'json'] }],
    })
    
    if (selected) {
      const template = await TemplateManager.importFromFile(selected as string)
      selectedTemplate.value = template
      showImport.value = false
    }
  } catch (error) {
    alert(`Failed to import template: ${error}`)
  }
}

async function importFromLink() {
  if (!importLink.value) return
  
  try {
    const template = await TemplateManager.importFromData(importLink.value)
    selectedTemplate.value = template
    showImport.value = false
    importLink.value = ''
  } catch (error) {
    alert(`Failed to import template: ${error}`)
  }
}

async function downloadSelected() {
  if (!selectedTemplate.value) return
  downloadTemplate(selectedTemplate.value)
}

async function copyLink() {
  if (!selectedTemplate.value) return
  
  try {
    await copyShareLink(selectedTemplate.value)
    alert('Link copied to clipboard!')
  } catch (error) {
    alert(`Failed to copy link: ${error}`)
  }
}

function useTemplate() {
  // Emit event or navigate to agent creation with template
  console.log('Using template:', selectedTemplate.value)
  alert(`Creating agent from template: ${selectedTemplate.value?.name}`)
  selectedTemplate.value = null
}

async function createTemplate() {
  try {
    const template: AgentTemplate = {
      schema: 'claw-template/v1',
      name: newTemplate.value.name,
      description: newTemplate.value.description,
      author: 'local-user',
      version: '1.0.0',
      created_at: new Date().toISOString(),
      system_prompt: newTemplate.value.system_prompt,
      model: newTemplate.value.model,
      computer_use: newTemplate.value.computer_use,
      sandbox_image: newTemplate.value.sandbox_image || undefined,
      hitl_level: newTemplate.value.hitl_level,
      tags: newTemplate.value.tags,
      channels: newTemplate.value.channels,
      max_tokens: undefined,
      temperature: undefined,
      budget_limit: undefined,
    }
    
    const link = await TemplateManager.exportTemplate({
      name: template.name,
      description: template.description,
      system_prompt: template.system_prompt,
      model: template.model,
      computer_use: template.computer_use,
      sandbox_image: template.sandbox_image,
      hitl_level: template.hitl_level,
      tags: template.tags,
      channels: template.channels,
    })
    
    // Copy to clipboard
    await navigator.clipboard.writeText(link)
    alert(`Template created! Share link copied to clipboard.\n\n${link}`)
    
    showCreate.value = false
    
    // Reset form
    newTemplate.value = {
      name: '',
      description: '',
      model: 'claude-3-5-sonnet-20241022',
      system_prompt: '',
      computer_use: false,
      sandbox_image: '',
      hitl_level: 'standard',
      tags: [],
      channels: [],
    }
  } catch (error) {
    alert(`Failed to create template: ${error}`)
  }
}
</script>

<style scoped>
.template-browser {
  padding: 1.5rem;
}

.browser-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.5rem;
}

.browser-header h2 {
  font-size: 1.25rem;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 0.75rem;
}

.btn-import, .btn-create {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-size: 0.875rem;
  cursor: pointer;
}

.btn-import {
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.3);
  color: #3b82f6;
}

.btn-create {
  background: #3b82f6;
  border: none;
  color: white;
}

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

.template-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 1.25rem;
  cursor: pointer;
  transition: all 0.2s;
}

.template-card:hover {
  border-color: rgba(59, 130, 246, 0.5);
  transform: translateY(-2px);
}

.template-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 0.5rem;
}

.template-name {
  font-weight: 600;
  font-size: 1rem;
}

.verified-badge {
  font-size: 0.875rem;
}

.template-author {
  font-size: 0.75rem;
  color: #888;
  margin-bottom: 0.75rem;
}

.template-desc {
  font-size: 0.875rem;
  color: #aaa;
  margin-bottom: 1rem;
  line-height: 1.5;
}

.template-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-bottom: 0.75rem;
}

.tag {
  padding: 0.25rem 0.5rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 4px;
  font-size: 0.75rem;
  color: #888;
}

.template-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: #666;
}

/* Modal styles */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: #1a1a2e;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  width: 90%;
  max-width: 600px;
  max-height: 85vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header h3 {
  font-size: 1.125rem;
  font-weight: 600;
}

.btn-close {
  background: none;
  border: none;
  color: #888;
  font-size: 1.5rem;
  cursor: pointer;
}

.modal-body {
  padding: 1.5rem;
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  gap: 0.75rem;
  justify-content: flex-end;
}

/* Drop zone */
.drop-zone {
  border: 2px dashed rgba(255, 255, 255, 0.2);
  border-radius: 12px;
  padding: 3rem;
  text-align: center;
  transition: all 0.2s;
}

.drop-zone.dragging {
  border-color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.drop-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.drop-text {
  color: #888;
  margin-bottom: 0.5rem;
}

.drop-or {
  color: #666;
  font-size: 0.875rem;
  margin: 1rem 0;
}

.btn-browse {
  padding: 0.5rem 1.5rem;
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 6px;
  color: #3b82f6;
  cursor: pointer;
}

.divider {
  text-align: center;
  position: relative;
  margin: 1.5rem 0;
}

.divider span {
  background: #1a1a2e;
  padding: 0 1rem;
  color: #666;
  font-size: 0.875rem;
}

.link-input {
  display: flex;
  gap: 0.5rem;
}

.input-field {
  flex: 1;
  padding: 0.75rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: white;
  font-size: 0.875rem;
}

.input-field:focus {
  outline: none;
  border-color: #3b82f6;
}

.btn-import-link {
  padding: 0.75rem 1.5rem;
  background: #3b82f6;
  border: none;
  border-radius: 6px;
  color: white;
  cursor: pointer;
}

.btn-import-link:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Detail modal */
.detail-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: #888;
  margin-top: 0.25rem;
}

.verified {
  color: #22c55e;
}

.detail-section {
  margin-bottom: 1.5rem;
}

.detail-section h4 {
  font-size: 0.875rem;
  font-weight: 500;
  color: #888;
  margin-bottom: 0.75rem;
}

.config-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.config-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.config-label {
  font-size: 0.75rem;
  color: #666;
}

.config-value {
  font-weight: 500;
}

.prompt-preview {
  background: rgba(0, 0, 0, 0.3);
  padding: 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.btn-download, .btn-copy {
  padding: 0.5rem 1rem;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: #888;
  cursor: pointer;
}

.btn-use {
  padding: 0.5rem 1.5rem;
  background: #22c55e;
  border: none;
  border-radius: 6px;
  color: white;
  cursor: pointer;
}

/* Create modal */
.form-group {
  margin-bottom: 1rem;
}

.form-group label {
  display: block;
  font-size: 0.875rem;
  color: #888;
  margin-bottom: 0.5rem;
}

.form-row {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
}

.prompt-input {
  min-height: 120px;
  resize: vertical;
}

.checkbox label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.btn-cancel {
  padding: 0.5rem 1rem;
  background: transparent;
  border: none;
  color: #888;
  cursor: pointer;
}

.btn-create-final {
  padding: 0.5rem 1.5rem;
  background: #3b82f6;
  border: none;
  border-radius: 6px;
  color: white;
  cursor: pointer;
}

.btn-create-final:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>