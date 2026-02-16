# 在线 API 配置指南

MaScribe 支持使用在线大模型 API（OpenAI 兼容格式）进行 AI 润色。

## 推荐默认配置（DeepSeek）

在 **设置 → AI Polishing Engine → Online API** 中填写：

```text
Endpoint: https://api.deepseek.com/v1
API Key:  （你的 DeepSeek Key）
Model:    deepseek-chat
```

> 程序会自动在 Endpoint 后拼接 `/chat/completions`，
> 你只需要填写到 `/v1`。

---

## 三个字段怎么填

| 字段 | 说明 | 示例 |
|------|------|------|
| Endpoint | API 基础地址（不含 `/chat/completions`） | `https://api.deepseek.com/v1` |
| API Key | 服务商提供的密钥 | `sk-xxxxxxxx` |
| Model | 模型名称 | `deepseek-chat` |

---

## 其他可用服务商（可选）

只要是 OpenAI 兼容接口，都可以接入。常见示例：

- DeepSeek：`https://api.deepseek.com/v1`
- Step-Fun：`https://api.stepfun.com/v1`
- Groq：`https://api.groq.com/openai/v1`
- OpenAI：`https://api.openai.com/v1`
- Qwen（DashScope 兼容）：`https://dashscope.aliyuncs.com/compatible-mode/v1`

---

## 快速排查

1. 日志提示 `API not configured`
- 三个字段（Endpoint / API Key / Model）必须都填写。

2. `401 Unauthorized`
- API Key 错误、过期，或未开通对应服务。

3. `404 Not Found`
- Endpoint 填写错误，或错误地包含了 `/chat/completions`。

4. 响应慢或效果不稳定
- 更换模型，或切换服务商节点。

---

## 提示词占位符

`AI Polish Prompt` 支持两个占位符：

- `{text}`：语音转写文本
- `{lang}`：识别语言代码

请保留 `{text}`，否则模型拿不到需要润色的内容。

