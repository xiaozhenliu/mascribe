# Online API Configuration Guide

MaScribe supports online LLM polishing through OpenAI-compatible APIs.

## Recommended Default (DeepSeek)

In **Settings → AI Polishing Engine → Online API**, use:

```text
Endpoint: https://api.deepseek.com/v1
API Key:  (your DeepSeek key)
Model:    deepseek-chat
```

> The app automatically appends `/chat/completions`.
> Only enter the base URL ending with `/v1`.

---

## What Each Field Means

| Field | Description | Example |
|------|-------------|---------|
| Endpoint | Base API URL (without `/chat/completions`) | `https://api.deepseek.com/v1` |
| API Key | Provider key | `sk-xxxxxxxx` |
| Model | Model name | `deepseek-chat` |

---

## Other Supported Providers (Optional)

Any OpenAI-compatible endpoint should work. Common examples:

- DeepSeek: `https://api.deepseek.com/v1`
- Step-Fun: `https://api.stepfun.com/v1`
- Groq: `https://api.groq.com/openai/v1`
- OpenAI: `https://api.openai.com/v1`
- Qwen (DashScope compatible): `https://dashscope.aliyuncs.com/compatible-mode/v1`

---

## Quick Troubleshooting

1. `API not configured`
- Fill all three fields: Endpoint / API Key / Model.

2. `401 Unauthorized`
- Invalid or expired API key, or service not enabled.

3. `404 Not Found`
- Incorrect Endpoint, or Endpoint incorrectly includes `/chat/completions`.

4. Slow or unstable results
- Try another model or provider endpoint.

---

## Prompt Placeholders

`AI Polish Prompt` supports:

- `{text}`: transcribed text
- `{lang}`: detected language code

Keep `{text}` in your prompt, otherwise the model receives no transcript input.

