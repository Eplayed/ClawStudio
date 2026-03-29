<script setup lang="ts">
/**
 * DockerGuide.vue - Docker 未安装引导横幅
 */
const emit = defineEmits<{
  close: []
  retry: []
}>()
</script>

<template>
  <div class="docker-guide show">
    <div class="docker-guide-header">
      <div class="dg-icon">🐳</div>
      <div class="dg-text">
        <h4>Docker 未安装或未运行</h4>
        <p>ClawStudio 需要 Docker 来创建隔离的沙盒环境。请先安装 Docker。</p>
      </div>
    </div>

    <ol class="docker-guide-steps">
      <li>
        <strong>Windows:</strong> 下载 <code>Docker Desktop for Windows</code>，
        安装后启动 Docker Desktop
      </li>
      <li>
        <strong>WSL2:</strong> 确保已启用 WSL2：<code>wsl --set-default-version 2</code>
      </li>
      <li>
        <strong>启动 Docker:</strong> 从开始菜单启动 <code>Docker Desktop</code>，
        等待左上角鲸鱼图标变为绿色
      </li>
      <li>
        <strong>验证:</strong> 打开 PowerShell 运行 <code>docker --version</code>
        确认版本号显示
      </li>
      <li>
        <strong>刷新:</strong> 点击下方「重新检测」按钮确认 Docker 已连接
      </li>
    </ol>

    <div class="docker-guide-actions">
      <button class="dg-btn primary" @click="emit('retry')">
        🔄 重新检测
      </button>
      <button class="dg-btn secondary" @click="emit('close')">
        关闭提示
      </button>
    </div>
  </div>
</template>

<style scoped>
.docker-guide {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.08), rgba(239, 68, 68, 0.03));
  border: 1px solid rgba(239, 68, 68, 0.25);
  border-radius: var(--radius);
  padding: 20px;
  margin-bottom: 20px;
}

.docker-guide-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 14px;
}

.dg-icon {
  width: 44px;
  height: 44px;
  border-radius: 10px;
  background: var(--red-glow);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  flex-shrink: 0;
}

.dg-text h4 {
  font-size: 14px;
  color: var(--red);
  margin-bottom: 2px;
}

.dg-text p {
  font-size: 11px;
  color: var(--text-secondary);
}

.docker-guide-steps {
  list-style: none;
  counter-reset: dg;
  margin: 0;
  padding: 0;
}

.docker-guide-steps li {
  counter-increment: dg;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid rgba(239, 68, 68, 0.1);
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.docker-guide-steps li:last-child {
  border-bottom: none;
}

.docker-guide-steps li::before {
  content: counter(dg);
  min-width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--red-glow);
  color: var(--red);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  font-family: var(--font-mono);
  flex-shrink: 0;
}

.docker-guide-steps li code {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--cyan);
}

.docker-guide-actions {
  margin-top: 14px;
  display: flex;
  gap: 10px;
}

.dg-btn {
  padding: 8px 18px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  font-family: var(--font-ui);
  border: none;
  transition: all 0.2s;
}

.dg-btn.primary {
  background: var(--cyan);
  color: #000;
}

.dg-btn.primary:hover {
  background: #08e8e8;
}

.dg-btn.secondary {
  background: var(--bg-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}

.dg-btn.secondary:hover {
  border-color: var(--border-light);
  color: var(--text-primary);
}
</style>
