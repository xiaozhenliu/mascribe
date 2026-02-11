# 在线 API 配置指南

Voice Input 支持使用在线大模型 API（OpenAI 兼容格式）来润色语音转写文本。
相比本地 1.5B 小模型，在线 API 在以下场景效果更好：

- 同音字纠错（如"云转文字"→"语音转文字"）
- 复杂长句整理
- 中英混合输入

## 配置方法

打开 **设置 → AI Polishing Engine → Online API**，填写三个字段：

| 字段 | 说明 | 示例 |
|------|------|------|
| **Endpoint** | API 基础地址（不含 `/chat/completions`） | `https://api.stepfun.com/v1` |
| **API Key** | 服务商提供的密钥 | `sk-xxxxxxxxxxxx` |
| **Model** | 模型名称 | `step-1-flash` |

> **注意**：程序会自动在 Endpoint 后面拼接 `/chat/completions`。
> 例如完整地址是 `https://api.example.com/v1/chat/completions`，
> 你只需要填写 `https://api.example.com/v1`。

---

## 推荐服务商

### 1. 阶跃星辰 (Step-Fun) — 中文推荐

- **官网**: https://platform.stepfun.com/
- **Endpoint**: `https://api.stepfun.com/v1`
- **推荐模型**: `step-1-flash`（速度快、价格低、中文效果好）
- **价格**: 约 ¥0.001/次（短文本非常便宜）
- **获取密钥**: 注册 → 控制台 → API 密钥 → 创建密钥

| 模型 | 速度 | 质量 | 价格 |
|------|------|------|------|
| `step-1-flash` | ★★★★★ | ★★★★ | ¥0.001/千 tokens |
| `step-2-16k` | ★★★ | ★★★★★ | ¥0.038/千 tokens |

### 2. 深度求索 (DeepSeek)

- **官网**: https://platform.deepseek.com/
- **Endpoint**: `https://api.deepseek.com/v1`
- **推荐模型**: `deepseek-chat`
- **价格**: ¥0.001/千 tokens（输入），¥0.002/千 tokens（输出）
- **获取密钥**: 注册 → API Keys → 创建

| 模型 | 速度 | 质量 | 价格 |
|------|------|------|------|
| `deepseek-chat` | ★★★★ | ★★★★★ | ¥0.001/千输入 |

### 3. 通义千问 (Alibaba Cloud Qwen)

- **官网**: https://dashscope.console.aliyun.com/
- **Endpoint**: `https://dashscope.aliyuncs.com/compatible-mode/v1`
- **推荐模型**: `qwen-turbo`
- **价格**: ¥0.0008/千 tokens
- **获取密钥**: 注册阿里云 → DashScope 控制台 → API Keys

| 模型 | 速度 | 质量 | 价格 |
|------|------|------|------|
| `qwen-turbo` | ★★★★★ | ★★★★ | ¥0.0008/千 tokens |
| `qwen-plus` | ★★★★ | ★★★★★ | ¥0.004/千 tokens |
| `qwen-max` | ★★★ | ★★★★★ | ¥0.02/千 tokens |

### 4. Groq — 英文优化，有免费额度

- **官网**: https://console.groq.com/
- **Endpoint**: `https://api.groq.com/openai/v1`
- **推荐模型**: `llama-3.1-8b-instant`
- **价格**: 免费额度（有频率限制）
- **获取密钥**: 注册 → API Keys → Create API Key

| 模型 | 速度 | 质量 | 价格 |
|------|------|------|------|
| `llama-3.1-8b-instant` | ★★★★★ | ★★★★ | 免费（有限流） |
| `llama-3.3-70b-versatile` | ★★★★ | ★★★★★ | 免费（有限流） |

### 5. OpenAI

- **官网**: https://platform.openai.com/
- **Endpoint**: `https://api.openai.com/v1`
- **推荐模型**: `gpt-4o-mini`
- **价格**: $0.15/百万输入 tokens，$0.60/百万输出 tokens
- **获取密钥**: 注册 → API Keys → Create new secret key

| 模型 | 速度 | 质量 | 价格 |
|------|------|------|------|
| `gpt-4o-mini` | ★★★★★ | ★★★★ | $0.15/百万输入 |
| `gpt-4o` | ★★★ | ★★★★★ | $2.50/百万输入 |

### 6. 硅基流动 (Silicon Flow) — 超低价

- **官网**: https://cloud.siliconflow.cn/
- **Endpoint**: `https://api.siliconflow.cn/v1`
- **推荐模型**: `Qwen/Qwen2.5-7B-Instruct`
- **价格**: ¥0.00035/千 tokens（极便宜）
- **获取密钥**: 注册 → API Keys → 创建

### 7. 智谱 AI (Zhipu)

- **官网**: https://open.bigmodel.cn/
- **Endpoint**: `https://open.bigmodel.cn/api/paas/v4`
- **推荐模型**: `glm-4-flash`
- **价格**: `glm-4-flash` 免费
- **获取密钥**: 注册 → API Keys

---

## 快速配置示例

### 最便宜（中文）：阶跃星辰

```
Endpoint: https://api.stepfun.com/v1
API Key:  （从 platform.stepfun.com 获取）
Model:    step-1-flash
```

### 免费（英文）：Groq

```
Endpoint: https://api.groq.com/openai/v1
API Key:  （从 console.groq.com 获取）
Model:    llama-3.1-8b-instant
```

### 免费（中文）：智谱 AI

```
Endpoint: https://open.bigmodel.cn/api/paas/v4
API Key:  （从 open.bigmodel.cn 获取）
Model:    glm-4-flash
```

### 最高质量：DeepSeek

```
Endpoint: https://api.deepseek.com/v1
API Key:  （从 platform.deepseek.com 获取）
Model:    deepseek-chat
```

---

## 自定义提示词

**AI Polish Prompt** 字段支持自定义模板，可使用以下占位符：

- `{text}` — 语音转写出的原始文本
- `{lang}` — SenseVoice 检测到的语言代码（`zh`、`en`、`ja`、`ko` 等）

### 默认提示词

```
Language: {lang}
You are a speech-to-text post-processor. Output ONLY the cleaned text, nothing else.

Rules:
1. Remove filler words (嗯、呃、那个、就是、然后、啊、um、uh、like、you know)
2. Remove repeated words and false starts
3. Fix punctuation and capitalization
4. Fix homophones based on context (同音字纠错，例如：云→语音、做→作、的→地/得)
5. Preserve ALL meaningful content — do not summarize, shorten, or rewrite
6. Do NOT translate, paraphrase, or add commentary
7. Keep the speaker's original meaning and sentence structure

{text}
```

### 自定义提示词技巧

- 提示词越短，响应越快
- 必须包含 `{text}`，这是语音文本插入的位置
- 中英混合场景下，可以在提示词中注明两种语言
- 如果使用中文优化的模型（如通义千问），可以用中文写提示词

---

## 常见问题

| 问题 | 解决方法 |
|------|----------|
| 日志显示 "API not configured" | 三个字段（Endpoint、API Key、Model）都要填写 |
| 超时错误 | 检查网络连接；换一个更快的模型 |
| 401 Unauthorized | API Key 不正确或已过期 |
| 404 Not Found | 检查 Endpoint 地址——不要包含 `/chat/completions` |
| 返回结果为空或异常 | 换一个模型试试；检查提示词模板 |
| 延迟太高（>2秒） | 换一个更快的模型或更近的服务商 |

---

## 费用估算

按照日常语音输入使用量（每次约 50-100 字，每天约 50 次）：

| 服务商 | 模型 | 日均费用 |
|--------|------|----------|
| 阶跃星辰 | step-1-flash | 约 ¥0.05 |
| DeepSeek | deepseek-chat | 约 ¥0.05 |
| 通义千问 | qwen-turbo | 约 ¥0.04 |
| Groq | llama-3.1-8b | 免费 |
| 智谱 | glm-4-flash | 免费 |
| OpenAI | gpt-4o-mini | 约 $0.01 |

语音润色非常便宜，因为每次请求都很短（通常不超过 200 tokens）。
