# hijack.py
import httpx
import requests
from urllib.parse import urlparse

TARGET_LLM_HOSTS =[
    'api.openai.com', 'api.anthropic.com', 'dashscope.aliyuncs.com',
    'api.deepseek.com', 'api.moonshot.cn', 'open.bigmodel.cn',
    'generativelanguage.googleapis.com', 'openai.azure.com'
]

PROXY_URL = "http://127.0.0.1:18788"

# --- 拦截 Requests 库 ---
original_request = requests.Session.request

def patched_request(self, method, url, **kwargs):
    parsed_url = urlparse(url)
    if any(host in parsed_url.hostname for host in TARGET_LLM_HOSTS if parsed_url.hostname):
        # 修改 URL 为本地代理
        new_url = f"{PROXY_URL}{parsed_url.path}?{parsed_url.query}"
        
        # 写入 Header
        headers = kwargs.get('headers', {})
        headers['x-claw-original-host'] = parsed_url.hostname
        headers['x-claw-original-url'] = url
        kwargs['headers'] = headers
        
        print(f"[🛡️ ClawStudio] 流量劫持 (Requests): {parsed_url.hostname} -> 127.0.0.1:18788")
        return original_request(self, method, new_url, **kwargs)
        
    return original_request(self, method, url, **kwargs)

requests.Session.request = patched_request
requests.request = patched_request

# --- 可以在这里继续拦截 httpx 或 aiohttp ---