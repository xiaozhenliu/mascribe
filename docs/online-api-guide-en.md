# Online API Configuration Guide

Voice Input supports using online LLM APIs (OpenAI-compatible format) for text polishing.
This provides higher quality results than the local 1.5B model, especially for:

- Homophone correction (同音字纠错)
- Complex sentence restructuring
- Multi-language mixed input

## How to Configure

Open **Settings → AI Polishing Engine → Online API**, then fill in three fields:

| Field | Description | Example |
|-------|-------------|---------|
| **Endpoint** | API base URL (without `/chat/completions`) | `https://api.stepfun.com/v1` |
| **API Key** | Your secret key from the provider | `sk-xxxxxxxxxxxx` |
| **Model** | Model name to use | `step-1-flash` |

> **Note**: The app automatically appends `/chat/completions` to the endpoint URL.
> So if the provider's full URL is `https://api.example.com/v1/chat/completions`,
> you only need to enter `https://api.example.com/v1`.

---

## Recommended Providers

### 1. Step-Fun (阶跃星辰) — Recommended for Chinese

- **Website**: https://platform.stepfun.com/
- **Endpoint**: `https://api.stepfun.com/v1`
- **Recommended Model**: `step-1-flash` (fast, cheap, great for Chinese)
- **Pricing**: ~¥0.001/request (very cheap for short text)
- **How to get API key**: Sign up → Console → API Keys → Create Key

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `step-1-flash` | ★★★★★ | ★★★★ | ¥0.001/1K tokens |
| `step-2-16k` | ★★★ | ★★★★★ | ¥0.038/1K tokens |

### 2. DeepSeek (深度求索)

- **Website**: https://platform.deepseek.com/
- **Endpoint**: `https://api.deepseek.com/v1`
- **Recommended Model**: `deepseek-chat`
- **Pricing**: ¥0.001/1K tokens (input), ¥0.002/1K tokens (output)
- **How to get API key**: Sign up → API Keys → Create new key

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `deepseek-chat` | ★★★★ | ★★★★★ | ¥0.001/1K input |

### 3. Alibaba Cloud Qwen (通义千问)

- **Website**: https://dashscope.console.aliyun.com/
- **Endpoint**: `https://dashscope.aliyuncs.com/compatible-mode/v1`
- **Recommended Model**: `qwen-turbo`
- **Pricing**: ¥0.0008/1K tokens
- **How to get API key**: Sign up for Aliyun → DashScope console → API Keys

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `qwen-turbo` | ★★★★★ | ★★★★ | ¥0.0008/1K tokens |
| `qwen-plus` | ★★★★ | ★★★★★ | ¥0.004/1K tokens |
| `qwen-max` | ★★★ | ★★★★★ | ¥0.02/1K tokens |

### 4. Groq (English-optimized, free tier)

- **Website**: https://console.groq.com/
- **Endpoint**: `https://api.groq.com/openai/v1`
- **Recommended Model**: `llama-3.1-8b-instant`
- **Pricing**: Free tier available (rate-limited)
- **How to get API key**: Sign up → API Keys → Create API Key

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `llama-3.1-8b-instant` | ★★★★★ | ★★★★ | Free (rate-limited) |
| `llama-3.3-70b-versatile` | ★★★★ | ★★★★★ | Free (rate-limited) |

### 5. OpenAI

- **Website**: https://platform.openai.com/
- **Endpoint**: `https://api.openai.com/v1`
- **Recommended Model**: `gpt-4o-mini`
- **Pricing**: $0.15/1M input tokens, $0.60/1M output tokens
- **How to get API key**: Sign up → API Keys → Create new secret key

| Model | Speed | Quality | Cost |
|-------|-------|---------|------|
| `gpt-4o-mini` | ★★★★★ | ★★★★ | $0.15/1M input |
| `gpt-4o` | ★★★ | ★★★★★ | $2.50/1M input |

### 6. Silicon Flow (硅基流动) — Budget option

- **Website**: https://cloud.siliconflow.cn/
- **Endpoint**: `https://api.siliconflow.cn/v1`
- **Recommended Model**: `Qwen/Qwen2.5-7B-Instruct`
- **Pricing**: ¥0.00035/1K tokens (extremely cheap)
- **How to get API key**: Sign up → API Keys → Create

### 7. Zhipu AI (智谱AI)

- **Website**: https://open.bigmodel.cn/
- **Endpoint**: `https://open.bigmodel.cn/api/paas/v4`
- **Recommended Model**: `glm-4-flash`
- **Pricing**: Free for `glm-4-flash`
- **How to get API key**: Sign up → API Keys

---

## Quick Setup Examples

### Cheapest (Chinese): Step-Fun

```
Endpoint: https://api.stepfun.com/v1
API Key:  (your key from platform.stepfun.com)
Model:    step-1-flash
```

### Free (English): Groq

```
Endpoint: https://api.groq.com/openai/v1
API Key:  (your key from console.groq.com)
Model:    llama-3.1-8b-instant
```

### Free (Chinese): Zhipu AI

```
Endpoint: https://open.bigmodel.cn/api/paas/v4
API Key:  (your key from open.bigmodel.cn)
Model:    glm-4-flash
```

### Best Quality: OpenAI / DeepSeek

```
Endpoint: https://api.deepseek.com/v1
API Key:  (your key from platform.deepseek.com)
Model:    deepseek-chat
```

---

## Custom Prompt

The **AI Polish Prompt** field accepts a custom template. Use these placeholders:

- `{text}` — the transcribed speech text
- `{lang}` — detected language code (`zh`, `en`, `ja`, `ko`, etc.)

### Default Prompt

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

### Tips for Custom Prompts

- Keep the prompt concise — shorter prompts = faster response
- Always include `{text}` — this is where your speech text gets inserted
- For bilingual scenarios, mention both languages in the prompt
- If using a Chinese-optimized model, you can write the prompt in Chinese

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| "API not configured" in logs | Fill in all three fields (Endpoint, API Key, Model) |
| Timeout errors | Check network connection; try a faster model |
| 401 Unauthorized | API key is incorrect or expired |
| 404 Not Found | Check endpoint URL — don't include `/chat/completions` |
| Empty or weird results | Try a different model; check prompt template |
| High latency (>2s) | Switch to a faster model or closer endpoint |

---

## Cost Estimate

For typical voice input usage (50-100 characters per utterance, ~50 uses/day):

| Provider | Model | Daily Cost |
|----------|-------|-----------|
| Step-Fun | step-1-flash | ~¥0.05 |
| DeepSeek | deepseek-chat | ~¥0.05 |
| Qwen | qwen-turbo | ~¥0.04 |
| Groq | llama-3.1-8b | Free |
| Zhipu | glm-4-flash | Free |
| OpenAI | gpt-4o-mini | ~$0.01 |

Voice input polishing is very cheap because each request is short (typically <200 tokens).
