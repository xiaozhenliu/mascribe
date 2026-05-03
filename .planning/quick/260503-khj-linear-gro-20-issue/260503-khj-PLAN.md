---
phase: quick-260503-khj
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/commands.rs
  - src-tauri/src/lib.rs
  - settings.html
  - src/settings.ts
autonomous: true
requirements: [GRO-20]

must_haves:
  truths:
    - "用户在设置页面选择'在线 API'模式时可以看到'测试连接'按钮"
    - "用户点击'测试连接'按钮后能看到连接测试结果"
    - "测试成功时显示响应时间"
    - "测试失败时显示详细错误信息"
  artifacts:
    - path: "src-tauri/src/commands.rs"
      provides: "test_online_api_connection 命令实现"
      exports: ["test_online_api_connection"]
    - path: "settings.html"
      provides: "测试连接按钮 UI"
      contains: "test-api-connection-btn"
    - path: "src/settings.ts"
      provides: "测试连接逻辑"
      contains: "testApiConnection"
  key_links:
    - from: "src/settings.ts"
      to: "test_online_api_connection"
      via: "invoke() 调用"
      pattern: "invoke.*test_online_api_connection"
    - from: "settings.html"
      to: "src/settings.ts"
      via: "按钮点击事件"
      pattern: "test-api-connection-btn.*click"
---

<objective>
为在线 AI API 配置添加连接测试功能,让用户能够验证 DeepSeek 等 API 配置是否正确。

Purpose: 解决用户配置 API 后无法确认配置正确性的问题
Output: 可用的"测试连接"按钮,显示测试结果和响应时间
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@/Users/xz/Projects/mascribe/CLAUDE.md
@/Users/xz/Projects/mascribe/.planning/STATE.md

# Existing code patterns
@/Users/xz/Projects/mascribe/src-tauri/src/commands.rs
@/Users/xz/Projects/mascribe/src-tauri/src/polishing/online.rs
@/Users/xz/Projects/mascribe/settings.html
@/Users/xz/Projects/mascribe/src/settings.ts
</context>

<tasks>

<task type="auto">
  <name>Task 1: 添加后端测试命令</name>
  <files>src-tauri/src/commands.rs, src-tauri/src/lib.rs</files>
  <action>
在 commands.rs 中添加 test_online_api_connection 命令:
- 接收参数: endpoint (String), api_key (String), model (String)
- 使用 OnlinePolisher::new() 创建实例
- 发送简单测试请求 (text="test", prompt_template="Reply: {text}", lang="en", screenshot=None)
- 返回 Result<TestConnectionResult, String>，其中 TestConnectionResult 包含 success (bool), response_time_ms (u64), error_message (Option<String>)
- 使用 std::time::Instant 测量响应时间
- 捕获所有错误并转换为友好的错误消息

在 lib.rs 的 invoke_handler 中注册 commands::test_online_api_connection
  </action>
  <verify>
    <automated>cargo check --manifest-path=/Users/xz/Projects/mascribe/src-tauri/Cargo.toml</automated>
  </verify>
  <done>命令编译通过,已在 invoke_handler 中注册</done>
</task>

<task type="auto">
  <name>Task 2: 添加前端测试按钮和逻辑</name>
  <files>settings.html, src/settings.ts</files>
  <action>
在 settings.html 的 api-settings section 中添加测试按钮:
- 在 detect-ollama-models-btn 的 api-field div 后添加新的 api-field div
- 包含 button#test-api-connection-btn.btn-small
- 添加 p#test-api-connection-hint.hint 用于显示测试结果

在 settings.ts 中实现测试逻辑:
- 添加 testApiConnection() async 函数
- 禁用按钮,显示"测试中..."
- 调用 invoke('test_online_api_connection', { endpoint, apiKey, model })
- 成功: 显示"✓ 连接成功 (响应时间: Xms)"
- 失败: 显示"✗ 连接失败: [错误信息]"
- 恢复按钮状态
- 在 setupApiConnectionTest() 中绑定点击事件
- 在 DOMContentLoaded 中调用 setupApiConnectionTest()
- 在 I18N 中添加相关文本 (test_connection, testing_connection, connection_success, connection_failed)
- 在 applyLanguage() 中应用翻译
  </action>
  <verify>
    <automated>npm run build --prefix /Users/xz/Projects/mascribe</automated>
  </verify>
  <done>前端编译通过,按钮仅在选择"在线 API"模式时可见</done>
</task>

<task type="auto">
  <name>Task 3: 手动测试验证</name>
  <files></files>
  <action>
启动开发环境测试功能:
1. 运行 npm run tauri dev
2. 打开设置页面
3. 选择"在线 API"模式
4. 填写测试 API 配置 (endpoint, key, model)
5. 点击"测试连接"按钮
6. 验证成功场景: 显示响应时间
7. 验证失败场景: 显示错误信息 (错误的 endpoint/key)
8. 验证按钮在测试期间被禁用
9. 切换到其他模式,确认按钮隐藏
  </action>
  <verify>
    <automated>echo "Manual testing completed - verify button works in both success and failure scenarios"</automated>
  </verify>
  <done>测试连接功能在各种场景下正常工作</done>
</task>

</tasks>

<verification>
- cargo check 通过
- npm run build 通过
- 手动测试: 测试按钮在"在线 API"模式下可见
- 手动测试: 成功场景显示响应时间
- 手动测试: 失败场景显示错误信息
- 手动测试: 测试期间按钮被禁用
</verification>

<success_criteria>
- 用户可以在设置页面测试在线 API 连接
- 测试成功时显示响应时间
- 测试失败时显示详细错误信息
- 按钮仅在"在线 API"模式下显示
- 测试过程中防止重复点击
</success_criteria>

<output>
After completion, create `.planning/quick/260503-khj-linear-gro-20-issue/260503-khj-SUMMARY.md`
</output>
