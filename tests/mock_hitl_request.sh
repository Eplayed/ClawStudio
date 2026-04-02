#!/bin/bash
# =============================================================================
# ClawStudio PRD v2.1 - 架构链路集成压测脚本
# =============================================================================
# 执行此脚本后，预期：
#   1. ClawStudio UI 会弹出工具拦截框 (HITL)
#   2. 右上角油表会增加相应费用
#   3. 如果用户在 UI 点拒绝，终端 curl 应收到含有 error 的响应
# =============================================================================

set -e

PROXY_URL="http://127.0.0.1:18788"
ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-sk-ant-demo}"

echo "=========================================="
echo "ClawStudio PRD v2.1 架构链路压测"
echo "=========================================="

# 检查 Proxy 是否运行
echo "[1/4] 检查 Proxy 网关状态..."
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" ${PROXY_URL}/health 2>/dev/null || echo "000")
if [ "$HTTP_CODE" != "200" ]; then
    echo "Proxy 未运行 (HTTP $HTTP_CODE)"
    echo "请先启动 ClawStudio 或运行: pnpm tauri dev"
    exit 1
fi
echo "Proxy 运行中 (HTTP $HTTP_CODE)"

# 模拟请求测试
echo ""
echo "[2/4] 发送测试请求..."

echo ""
echo "[3/4] 通过 Proxy 发送 Anthropic 格式请求..."

RESPONSE=$(curl -s -X POST "${PROXY_URL}/v1/messages" \
  -H "Content-Type: application/json" \
  -H "x-api-key: ${ANTHROPIC_API_KEY}" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 1024,
    "messages": [
      {
        "role": "user",
        "content": "Hello, test the proxy"
      }
    ]
  }' 2>&1)

echo "响应: $RESPONSE"

# 检查响应
echo ""
echo "[4/4] 验证响应..."
if echo "$RESPONSE" | grep -qi "error"; then
    echo "收到错误响应"
elif echo "$RESPONSE" | grep -qi "circuit"; then
    echo "收到熔断响应"
elif echo "$RESPONSE" | grep -qi "content"; then
    echo "收到正常响应"
else
    echo "收到未知响应"
fi

echo ""
echo "=========================================="
echo "压测完成"
echo "=========================================="
echo ""
echo "验证前端弹窗，请在浏览器中打开 http://localhost:1420"
echo "验证费用追踪，检查右上角 FuelGauge"
echo ""
