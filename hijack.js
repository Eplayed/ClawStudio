/**
 * ClawStudio - Qwen Portal Hijack Script (Monitor-Only)
 * 
 * 猴子补丁：拦截所有发往 qwen-portal 的请求并记录
 * 用法: NODE_OPTIONS="--require /path/to/hijack_qwen.js" qclaw
 * 
 * 注意：这个脚本只做监控，不影响正常请求
 */

const originalFetch = global.fetch;
const originalWebSocket = global.WebSocket;

// ClawStudio 监控端点
const MONITOR_ENDPOINT = 'http://127.0.0.1:18788/monitor';

// 🎯 全球主流大模型 API 域名/特征清单
const TARGET_LLM_HOSTS =[
    // --- 国际主流 ---
    'api.openai.com',                     // OpenAI (GPT 系列)
    'api.anthropic.com',                  // Anthropic (Claude 系列)
    'generativelanguage.googleapis.com',  // Google Gemini
    'api.mistral.ai',                     // Mistral
    'api.cohere.ai',                      // Cohere
    'api.groq.com',                       // Groq (极速推理)
    'openai.azure.com',                   // Azure OpenAI (匹配所有子域名)
    
    // --- 国内主流 ---
    'dashscope.aliyuncs.com',             // 阿里云 通义千问 (Qwen)
    'api.deepseek.com',                   // DeepSeek (深度求索)
    'api.moonshot.cn',                    // Moonshot (Kimi)
    'open.bigmodel.cn',                   // 智谱 AI (GLM 系列)
    'api.baichuan-ai.com',                // 百川智能
    'aip.baidubce.com',                   // 百度 文心一言 (ERNIE)
    'spark-api.xf-yun.com'                // 讯飞星火
];

// 判断是否需要拦截
function shouldIntercept(url) {
  const urlStr = typeof url === 'string' ? url : url.toString();
  return TARGET_LLM_HOSTS.some(host => urlStr.includes(host));
}

// 发送到 ClawStudio 监控（异步，不阻塞）
async function sendToMonitor(event) {
  try {
    fetch(MONITOR_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(event),
    }).catch(() => {}); // 静默失败
  } catch (e) {
    // 忽略所有错误
  }
}

// 解析请求体
function parseBody(body, options) {
  if (!body) return null;
  try {
    if (typeof body === 'string') {
      return JSON.parse(body);
    }
    if (body instanceof URLSearchParams) {
      return Object.fromEntries(body);
    }
    if (typeof body === 'object') {
      return body;
    }
  } catch (e) {}
  return null;
}

// 劫持 fetch
global.fetch = async function hijackedFetch(url, options = {}) {
  const urlStr = typeof url === 'string' ? url : (url instanceof URL ? url.toString() : String(url));
  
  // 检查是否需要拦截
  if (!shouldIntercept(urlStr)) {
    return originalFetch(url, options);
  }
  
  const startTime = Date.now();
  
  // 解析请求
  const parsedBody = parseBody(options?.body);
  const requestEvent = {
    type: 'llm_request',
    timestamp: new Date().toISOString(),
    method: options?.method || 'GET',
    url: urlStr,
    model: parsedBody?.model || 'unknown',
    messageCount: parsedBody?.messages?.length || 0,
    stream: parsedBody?.stream || false,
  };
  
  // 发送请求事件（不等待）
  sendToMonitor(requestEvent);
  
  try {
    // 执行真实请求
    const response = await originalFetch(url, options);
    
    // 计算耗时
    const duration = Date.now() - startTime;
    
    // 克隆响应以读取
    const responseClone = response.clone();
    
    // 尝试读取响应体
    responseClone.text().then(text => {
      try {
        const data = JSON.parse(text);
        const responseEvent = {
          type: 'llm_response',
          timestamp: new Date().toISOString(),
          url: urlStr,
          duration,
          status: response.status,
          model: data.model || parsedBody?.model || 'unknown',
          usage: data.usage || null,
          finishReason: data.choices?.[0]?.finish_reason || null,
        };
        sendToMonitor(responseEvent);
      } catch (e) {
        // 非 JSON 响应
        const responseEvent = {
          type: 'llm_response',
          timestamp: new Date().toISOString(),
          url: urlStr,
          duration,
          status: response.status,
          error: 'Failed to parse response',
        };
        sendToMonitor(responseEvent);
      }
    }).catch(() => {});
    
    return response;
    
  } catch (error) {
    // 错误事件
    sendToMonitor({
      type: 'llm_error',
      timestamp: new Date().toISOString(),
      url: urlStr,
      error: error.message,
      stack: error.stack,
    });
    throw error;
  }
};

// 劫持 WebSocket（用于流式响应）
let wsHijacked = false;
try {
  if (typeof WebSocket !== 'undefined') {
    const OriginalWS = WebSocket;
    
    global.WebSocket = class HijackedWebSocket extends OriginalWS {
      constructor(url, ...args) {
        const urlStr = typeof url === 'string' ? url : url.toString();
        
        if (shouldIntercept(urlStr)) {
          sendToMonitor({
            type: 'llm_ws_connect',
            timestamp: new Date().toISOString(),
            url: urlStr,
          });
        }
        
        super(url, ...args);
        
        this.addEventListener('message', (event) => {
          try {
            const data = JSON.parse(event.data);
            if (data.choices || data.model || data.token) {
              sendToMonitor({
                type: 'llm_ws_message',
                timestamp: new Date().toISOString(),
                url: urlStr,
                data: data,
              });
            }
          } catch (e) {}
        });
      }
    };
    
    // 复制静态属性
    global.WebSocket.CONNECTING = OriginalWS.CONNECTING;
    global.WebSocket.OPEN = OriginalWS.OPEN;
    global.WebSocket.CLOSING = OriginalWS.CLOSING;
    global.WebSocket.CLOSED = OriginalWS.CLOSED;
    
    wsHijacked = true;
  }
} catch (e) {
  // WebSocket 劫持失败，可能环境不支持
}

console.log('╔═══════════════════════════════════════════════════════════╗');
console.log('║           ClawStudio Qwen Portal Hijack Loaded            ║');
console.log('╠═══════════════════════════════════════════════════════════╣');
console.log('║ Monitoring LLM endpoints:                                 ║');
TARGET_LLM_HOSTS.forEach(host => {
  console.log('║   • ' + host.padEnd(53) + '║');
});
console.log('║                                                           ║');
console.log('║ Events will be sent to: http://127.0.0.1:18788/monitor   ║');
console.log('║                                                           ║');
console.log('║ To use with QClaw:                                        ║');
console.log('║   NODE_OPTIONS="--require /path/to/hijack_qwen.js" qclaw  ║');
console.log('╚═══════════════════════════════════════════════════════════╝');
