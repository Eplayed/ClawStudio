# ClawStudio (Nova) — PRD 差异补全任务书

## ✅ 项目完成度

**35 / 35 功能点 = 100%** ✨

所有 7 项补全任务已完成！

---

## 补全任务一览

| # | 功能 | 位置 | 优先级 | 难度 |
|---|------|------|--------|------|
| D1 | Dashboard 最近动态时间线 | Dashboard.vue | P1 | 小 |
| D2 | Dashboard 7 天费用趋势图 | Dashboard.vue | P1 | 小 |
| D3 | Cost Monitor 页面（油表 + Token 分解 + 费用明细 + 自动熔断） | 新页面 CostMonitor.vue | P0 | 大 |
| D4 | 成本引擎接入 Store（budget-exceeded 自动停止） | agents.ts | P0 | 中 |
| D5 | Sandboxes 本机/Docker 运行模式切换 | Sandboxes.vue | P2 | 小 |
| D6 | Sandboxes CPU/Memory 实时 mini-charts | SandboxCard.vue | P2 | 中 |
| D7 | Sandboxes 可视化配置编辑器 | Sandboxes.vue 或 Environment 区块 | P2 | 小 |

建议执行顺序：**D3 → D4 → D1 → D2 → D5 → D6 → D7**

---

## D1. Dashboard — 最近动态时间线

**目标：** 在 Dashboard.vue 的 Agent Grid 下方添加「最近动态 Recent Activity」面板。

**需要修改的文件：**
- `src/views/Dashboard.vue`

**参考原型：** `clawstudio.html` 中 `class="activity-list"` 和 `class="activity-item"` 部分（约第 2396-2425 行）

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio，Tauri 桌面应用，暗色赛博主题。

请在 src/views/Dashboard.vue 中添加「最近动态 Recent Activity」区域。要求：

1. 在现有 agent-grid 之后添加新的 section-header + 面板
2. 面板内是一个时间线列表，每条包含：
   - 时间戳（左侧，monospace 小字，如 14:32）
   - 图标（中间，如 ✅ 🚨 💬 🔍 📸）
   - 描述文本（右侧，高亮 Agent 名称为 bold）
3. 目前用 mock 数据（5-6 条），数据结构：
   interface ActivityItem {
     time: string
     icon: string
     text: string   // 支持 <strong> 标签高亮
   }
4. 样式：
   - 每行 padding: 8px 0，border-bottom: 1px solid var(--border)
   - 时间戳 color: var(--text-dim), font-family: var(--font-mono), font-size: 10px, min-width: 50px
   - 描述 color: var(--text-secondary), strong 用 var(--text-primary)
   - 面板整体 background: var(--bg-card), border: 1px solid var(--border), border-radius: var(--radius), padding: 16px
5. 未来接入：替换 mock 为 useAgentStore().eventLog 的最近 N 条（加注释即可）

参考 CSS 变量：src/styles/variables.css
```

---

## D2. Dashboard — 7 天费用趋势图

**目标：** 在 Dashboard.vue 添加近 7 天 API 费用柱状图。

**需要修改的文件：**
- `src/views/Dashboard.vue`

**参考原型：** `clawstudio.html` 中 `class="cost-bars"` 和 `class="cost-bar-col"` 部分（约第 2419-2437 行）

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio。

请在 src/views/Dashboard.vue 中添加「费用趋势 Cost Trend」面板，与「最近动态」并排（两列布局）。

1. 面板标题：📈 费用趋势 Cost Trend (7天)
2. 内容为纯 CSS 柱状图（不使用图表库），7 根竖条代表周一至周日
3. 每根竖条：
   - 容器高度固定 100px，竖条从底部向上生长
   - 竖条 width: 28px, border-radius: 4px 4px 0 0
   - 默认渐变 background: linear-gradient(180deg, var(--cyan), var(--cyan-dim))
   - 「今日」竖条用琥珀色 linear-gradient(180deg, var(--amber), var(--amber-dim))
   - 竖条上方悬浮 hover 显示金额 tooltip（::before 或 :hover + ::after）
4. 底部显示星期标签（周一 ~ 今日），font-size: 10px, color: var(--text-dim)
5. Mock 数据：
   const costData = [
     { day: '周一', cost: 1.23 },
     { day: '周二', cost: 1.50 },
     { day: '周三', cost: 1.15 },
     { day: '周四', cost: 1.87 },
     { day: '周五', cost: 2.04 },
     { day: '周六', cost: 0.55 },
     { day: '今日', cost: 2.47, highlight: true },
   ]
   竖条高度 = (cost / maxCost) * 100 + 'px'
6. 面板与 D1「最近动态」面板在同一行用 grid 两列布局：
   display: grid; grid-template-columns: 1fr 1fr; gap: 14px;
7. 响应式 @media (max-width: 900px) 改为单列

参考 CSS 变量：src/styles/variables.css
```

---

## D3. Cost Monitor 页面 — 烧钱计算器（核心缺失页面）

**目标：** 创建新的 CostMonitor.vue 页面，包含预算油表、Token 分解条形图、Per-Agent 费用明细表、自动熔断滑条。

**需要创建的文件：**
- `src/views/CostMonitor.vue`
- `src/components/FuelGauge.vue`
- `src/components/TokenBreakdown.vue`
- `src/components/AgentCostTable.vue`

**需要修改的文件：**
- `src/router/index.ts` — 添加 `/cost` 路由
- `src/components/Sidebar.vue` — 在 navItems 中添加 Cost Monitor 项

**参考原型：** `clawstudio.html` 中 `class="fuel-gauge-section"` 整个区块（约第 2441-2511 行），包含 SVG 油表、Token 条形图、Agent 费用表

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio，暗色赛博主题 Tauri 桌面应用。

请创建「烧钱计算器 Cost Monitor」页面及其子组件。这是 PRD 规定但尚未实现的页面。

=== 步骤 1：修改路由和导航 ===

1. 在 src/router/index.ts 中添加路由：
   {
     path: '/cost',
     name: 'cost',
     component: () => import('@/views/CostMonitor.vue'),
     meta: { title: '烧钱计算器', icon: '💰' },
   }
   放在 /overwatch 和 /sandboxes 之间

2. 在 src/components/Sidebar.vue 的 navItems 中添加：
   { path: '/cost', name: 'cost', label: '烧钱计算器', icon: '💰', section: 'main' }
   放在「监控舱」之后

=== 步骤 2：创建 FuelGauge.vue ===

Props: current (number, 当前费用), limit (number, 预算上限)

功能：
- 圆形 SVG 仪表盘：viewBox="0 0 180 180"
- 底层灰色环 track：<circle cx="90" cy="90" r="75" stroke="var(--border)" stroke-width="10" fill="none">
- 上层渐变填充弧 fill-arc：stroke-dasharray="471"，stroke-dashoffset 根据 (1 - current/limit) * 471 计算
- SVG linearGradient：#22c55e → #f0a030 → #ef4444
- 中心文字叠加层（绝对定位）：
  - 金额显示：$4.72（大号数字，颜色根据占比变化）
  - 上限：/ $10.00 上限（小号灰色）
  - 百分比 badge：47.2%（带圆角背景）
- 底部图例：安全(绿) / 警告(琥珀) / 危险(红) 三色圆点
- 颜色逻辑：0-50% green, 50-80% amber, 80%+ red
- 使用 src/utils/costCalculator.ts 中的 getBudgetStatus / getBudgetColor

参考 HTML：clawstudio.html 中 class="fuel-gauge-wrap" 和 class="fuel-gauge"（第 2442-2467 行）

=== 步骤 3：创建 TokenBreakdown.vue ===

Props: inputTokens (number), outputTokens (number), imageTokens (number)

功能：
- 三条水平进度条，每条包含：
  - 标签（左侧）：📥 Input Tokens / 📤 Output Tokens / 📷 Image Tokens (截图)
  - 进度条轨道 background: var(--border), 填充色分别为 cyan / green / amber
  - 百分比标签在填充条内部，数值在右侧
- 底部总计行：总计 Total Tokens + 数字
- 百分比计算：各类占总量的比例
- 数字使用 font-family: var(--font-mono) 格式化为千分位

参考 HTML：clawstudio.html 中 class="token-breakdown" 和 class="token-bars"（第 2469-2497 行）

=== 步骤 4：创建 AgentCostTable.vue ===

Props: agents (Agent[] 从 store 获取)

功能：
- 表格列：特工 | 任务 | Tokens | 消耗 | 占比
- 消耗列颜色高亮（用 formatCost 函数）
- 占比列用百分比 + 小进度条
- 数据从 useAgentStore().agents 读取，使用 currentCost / tokensUsed 字段
- 空状态显示「暂无活跃特工」

参考 HTML：clawstudio.html 中 class="agent-cost-table"（第 2500-2510 行）

=== 步骤 5：创建 CostMonitor.vue ===

组合以上三个组件：
- 顶栏：💰 烧钱计算器 Cost Monitor
- 布局：grid 两列
  - 左列 280px：FuelGauge
  - 右列 flex: 1：TokenBreakdown + AgentCostTable
- 底部区块：自动熔断设置
  - 标题：⚡ 自动熔断 Auto Cutoff
  - range 滑条：$0.50 ~ $50.00，step 0.50
  - 当前值显示
  - 说明文字：当任意 Agent 消耗超过此上限时自动暂停并弹出通知
  - v-model 绑定到 useSettingsStore().budgetDefault

CSS 使用 scoped，复用 variables.css 变量。整体风格与 Dashboard 统一。

参考：
- clawstudio.html 中 fuel-gauge-section 完整区块
- src/utils/costCalculator.ts（getBudgetStatus, formatCost, calculateCost）
- src/stores/agents.ts（agents 数组和 totalCostToday）
- src/stores/settings.ts（budgetDefault）
```

---

## D4. 成本引擎接入 Store — budget-exceeded 自动停止

**目标：** 将 costCalculator.ts 接入 agents store，实现费用实时累加和预算超限自动停止。

**需要修改的文件：**
- `src/stores/agents.ts`

**AI 提示词：**

```
你是一个 TypeScript 开发者。项目是 ClawStudio。

当前 src/stores/agents.ts 的 initEventListener 监听 'openclaw-event' 并将事件推入 eventLog。
当前 src/utils/eventParser.ts 已实现 parseOpenClawEvent 和 extractCursorPosition。
当前 src/utils/costCalculator.ts 已实现 calculateCost。

需要修改 src/stores/agents.ts：

1. 在 eventListener 中调用 parseOpenClawEvent 解析 raw 事件：
   import { parseOpenClawEvent, extractCursorPosition } from '@/utils/eventParser'
   import { calculateCost } from '@/utils/costCalculator'

2. 将 eventLog 的类型改为 ParsedEvent[]（而非 OpenClawEvent[]）：
   import type { ParsedEvent } from '@/utils/eventParser'
   const eventLog = ref<ParsedEvent[]>([])

3. 在 listener 中：
   const parsed = parseOpenClawEvent(data.raw || JSON.stringify(data))
   eventLog.value.push(parsed)

4. 新增 ref：
   const cursorPosition = ref<{ x: number; y: number }>({ x: 0, y: 0 })

5. 如果解析出 mouse_move 坐标，更新 cursorPosition：
   const pos = extractCursorPosition(parsed)
   if (pos) cursorPosition.value = pos

6. 如果解析出 tokenUsage，累加到对应 agent 的 tokensUsed 和 currentCost：
   if (parsed.tokenUsage) {
     const agent = agents.value.find(a => a.id === data.agent_id)
     if (agent) {
       agent.tokensUsed += parsed.tokenUsage.input + parsed.tokenUsage.output + parsed.tokenUsage.image
       const model = useSettingsStore().defaultModel
       agent.currentCost += calculateCost(model, parsed.tokenUsage)
       // Budget check
       if (agent.currentCost >= agent.budgetLimit && agent.budgetLimit > 0) {
         stopAgent(agent.id)
         console.warn(`[Budget Exceeded] ${agent.name}: $${agent.currentCost.toFixed(4)} >= $${agent.budgetLimit}`)
         // 未来: emit 前端通知事件
       }
     }
   }

7. 在 return 中导出 cursorPosition

8. 确保 eventLog 的 bounded 清理逻辑仍然生效（> 500 条时保留最近 300 条）

不要改变现有的 startAgent / stopAgent / addAgent / removeAgent 接口。
保持向后兼容：eventLog 中既可以有 ParsedEvent 也可以有旧格式 push 的数据。
```

---

## D5. Sandboxes — 本机/Docker 运行模式切换

**目标：** 在 Sandboxes.vue 顶部添加「本机运行 / Docker 沙盒运行」模式切换开关。

**需要修改的文件：**
- `src/views/Sandboxes.vue`
- `src/stores/settings.ts` — 新增 runMode ref

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio。

请在 src/views/Sandboxes.vue 和 src/stores/settings.ts 中添加运行模式切换功能。

1. 在 src/stores/settings.ts 中：
   - 新增 ref: const runMode = ref<'local' | 'docker'>('docker')
   - 在 loadAllSettings 的 switch 中添加 case 'run_mode'
   - 在 saveAllSettings 的 entries 中添加 ['run_mode', runMode.value]
   - 在 return 中导出 runMode

2. 在 src/views/Sandboxes.vue 的 topbar 和 sandbox-content 之间添加模式切换面板：
   - 两个大按钮（Segmented Control 样式）：
     - 🖥 本机运行 (Local)：直接使用本机环境，无 Docker 隔离
     - 🐳 Docker 沙盒 (Docker)：在隔离容器中运行，更安全
   - 选中状态用 cyan 高亮 + border + glow
   - 未选中状态用 var(--bg-card) + var(--text-dim)
   - 切换时更新 settingsStore.runMode 并保存
   - 选择「本机运行」时下方显示一行警告：
     ⚠️ 本机模式下 Agent 直接操作你的系统，请确保 HITL 审批已开启
     使用 amber 背景警告样式

3. 当 runMode === 'local' 时：
   - 隐藏 Docker 状态面板和沙盒列表
   - 显示一个简单的状态面板：
     - 运行环境：本机
     - Node.js 版本（mock: v20.11.0）
     - Python 版本（mock: 3.12.1）
     - HITL 状态：已启用/已禁用（读取 settingsStore.hitlEnabled）

4. 当 runMode === 'docker' 时保持现有逻辑不变
```

---

## D6. SandboxCard — CPU/Memory 实时 mini-charts

**目标：** 在 SandboxCard 中显示容器 CPU 和内存的实时占用条。

**需要修改的文件：**
- `src/components/SandboxCard.vue`
- `src/views/Sandboxes.vue` — 添加 stats 轮询

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio。

需要在 SandboxCard 中添加 CPU 和内存实时占用显示，并在 Sandboxes.vue 中添加定时轮询。

=== 步骤 1：修改 Sandboxes.vue ===

1. 新增 ref：
   const sandboxStats = ref<Record<string, { cpu_percent: number; memory_used_mb: number; memory_limit_mb: number }>>({})

2. 创建轮询函数 pollStats()：
   - 遍历 sandboxes.value 中 status === 'running' 的沙盒
   - 对每个调用 invoke('get_sandbox_stats', { container_id: sb.id })
   - 将结果存入 sandboxStats[sb.id]
   - 错误时忽略（容器可能刚停止）

3. onMounted 中启动 setInterval(pollStats, 5000)，onUnmounted 中清除

4. 将 sandboxStats 作为 prop 或通过 :stats="sandboxStats[sb.id]" 传给 SandboxCard

=== 步骤 2：修改 SandboxCard.vue ===

1. 新增 prop：stats?: { cpu_percent: number; memory_used_mb: number; memory_limit_mb: number }

2. 在卡片中添加资源监控区域（仅当 stats 存在时显示）：
   - CPU 行：
     标签 "CPU" + 百分比数字 + 水平进度条
     进度条颜色：< 50% green, 50-80% amber, > 80% red
   - Memory 行：
     标签 "MEM" + "1234 MB / 1024 MB" + 水平进度条
     进度条颜色逻辑同上

3. 进度条样式：
   - 轨道 height: 4px, background: var(--border), border-radius: 2px
   - 填充 transition: width 0.5s ease，颜色根据占比切换
   - 数字 font-family: var(--font-mono), font-size: 10px

4. 无 stats 时显示灰色文字「等待数据…」

参考 Rust 后端 get_sandbox_stats 返回的结构：
{ cpu_percent: f64, memory_used: u64, memory_limit: u64 }
其中 memory 单位是 bytes，前端需要转 MB。
```

---

## D7. Sandboxes — 可视化配置编辑器区块

**目标：** 在 Sandboxes.vue 底部添加快捷配置编辑器（API Key、Model、System Prompt）。

**需要修改的文件：**
- `src/views/Sandboxes.vue`

**AI 提示词：**

```
你是一个 Vue3 + TypeScript 前端开发者。项目是 ClawStudio。

PRD 要求在 Environment 页面中包含可视化配置编辑器。当前 Sandboxes 页面对应 Environment 模块。
完整的配置在 Settings 页面已有，此处只放常用快捷配置入口。

请在 Sandboxes.vue 底部（sandbox-grid 之后）添加一个「快捷配置」面板：

1. 面板标题：🔧 快捷配置 Quick Config

2. 三列网格布局（响应式改两列或单列），每列一个配置块：

   块 1 — 当前模型：
   - 显示当前选中的模型名（settingsStore.defaultModel）
   - 一个 <select> 下拉框快速切换模型
   - 选项：Claude 3.5 Sonnet / Claude 3 Opus / GPT-4o
   - 修改后自动调用 settingsStore.saveAllSettings()

   块 2 — API Key 状态：
   - 显示 Claude Key 状态：✅ 已验证 或 ❌ 未配置
   - 显示 OpenAI Key 状态：✅ 已验证 或 ❌ 未配置
   - 读取 settingsStore.claudeKeyValid / openaiKeyValid
   - 底部一个「前往设置」链接按钮跳转到 /settings

   块 3 — HITL 开关：
   - 显示当前 HITL 状态
   - 一个 toggle 开关控制 settingsStore.hitlEnabled
   - 修改后自动保存

3. 每个配置块样式：
   background: var(--bg-card), border: 1px solid var(--border), border-radius: var(--radius), padding: 16px

4. 整体容器 margin-top: 24px
```

---

## 完成后的验证检查清单

完成以上 7 项后，对照 PRD 逐项验证：

- [ ] Dashboard：4 统计卡 + Agent 网格 + 最近动态 + 7 天费用图
- [ ] Agents：看板 + 通道聚合 + 创建 Modal (CU/沙盒联动)
- [ ] Overwatch：思维流 + 视觉流 (模拟+VNC) + HITL 三态审批
- [ ] Cost Monitor：油表 + Token 分解 + 费用明细 + 自动熔断
- [ ] Sandboxes：Docker 状态 + 沙盒管理 + 本机/Docker 切换 + CPU/MEM 图 + 快捷配置
- [ ] Traces：回放播放器 (速度/导出) + 历史表格
- [ ] Settings：密钥 + 模型 + 温度 + HITL + 网络 + 持久化
- [ ] Store 集成：eventParser → agents store → cursorPosition / cost 累加 / 自动停止
- [ ] 侧边栏：新增 Cost Monitor 导航项，共 7 个导航入口

---

## 补充说明

### 新增路由后的导航结构

```
Main
  📊 仪表盘        /
  🤖 特工列队       /agents
  📺 监控舱        /overwatch
  💰 烧钱计算器     /cost          ← 新增 (D3)
System
  📦 沙盒环境       /sandboxes
  ⏱ 历史回放       /traces
  ⚙ 系统设置       /settings
```

### 文件变更汇总

| 操作 | 文件路径 |
|------|----------|
| 新建 | `src/views/CostMonitor.vue` |
| 新建 | `src/components/FuelGauge.vue` |
| 新建 | `src/components/TokenBreakdown.vue` |
| 新建 | `src/components/AgentCostTable.vue` |
| 修改 | `src/views/Dashboard.vue` — 添加 D1 + D2 |
| 修改 | `src/views/Sandboxes.vue` — 添加 D5 + D6 轮询 + D7 |
| 修改 | `src/components/SandboxCard.vue` — 添加 D6 资源条 |
| 修改 | `src/stores/agents.ts` — 添加 D4 成本引擎 |
| 修改 | `src/stores/settings.ts` — 添加 runMode (D5) |
| 修改 | `src/router/index.ts` — 添加 /cost 路由 (D3) |
| 修改 | `src/components/Sidebar.vue` — 添加导航项 (D3) |
