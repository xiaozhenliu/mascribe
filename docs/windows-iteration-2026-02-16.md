# Windows 适配迭代文档（2026-02-16）

## 1. 目标

本迭代聚焦于把 Windows 适配从“后端已具备”推进到“用户可感知一致”：

1. 修复设置页中的平台文案不一致（此前偏 macOS）。
2. 让 OCR 原生选项和提示文案按平台动态显示。
3. 更新迁移状态文档，移除已完成但未同步的待办项。

---

## 2. 范围

### In Scope

- Tauri 后端提供运行平台查询命令。
- 设置页（`settings.html` / `src/settings.ts`）改为平台感知文案。
- 迁移文档状态更新。

### Out of Scope

- Windows 真机构建与安装包验证。
- Phase 2 的本地 Vision 模型后端实现。

---

## 3. 执行计划

1. 新增 `get_platform` 命令并注册到 `invoke_handler`。
2. 前端加载时读取平台，动态更新：
   - Native OCR 单选项标签（macOS / Windows）。
   - Native OCR 说明文案。
   - Screenshot 保存路径提示。
3. 更新 `docs/windows-migration-plan.md` 当前状态。
4. 运行本地可执行检查命令，记录结果。

---

## 4. 实施记录

### 4.1 已实施改动

- 后端：
  - `src-tauri/src/commands.rs`：新增 `get_platform()`。
  - `src-tauri/src/lib.rs`：注册 `commands::get_platform`。
- 前端：
  - `settings.html`：原生 OCR 标签增加 `id="vision-native-label"`，由脚本动态赋值。
  - `src/settings.ts`：
    - 新增平台检测逻辑（调用 `invoke("get_platform")`）。
    - 新增原生 OCR 标签动态更新。
    - 将 Screenshot 保存路径提示改为平台分支。
    - 将 Native OCR 说明改为平台分支。
- 文档：
  - `docs/windows-migration-plan.md`：将“前端设置界面更新”从待完成移入已完成。

### 4.2 验证结果

- 已尝试执行 Windows 目标检查：
  - 命令：`cargo check --target x86_64-pc-windows-msvc`
  - 结果：失败（当前环境无法解析 `static.crates.io`，非代码错误）。

---

## 5. 下一步

1. 在 Windows 真机完成 `cargo check` / `cargo tauri build`（MSVC 目标）。
2. 安装包侧验证（`msi` / `nsis`）和首次运行权限行为回归。
3. Phase 2：Vision 本地模型后端选型与落地（ONNX Runtime 或 llama.cpp vision）。
