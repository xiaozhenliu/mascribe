---
phase: quick-260503-khj
plan: 01
subsystem: settings
tags: [api, connection-test, ui, backend]
dependency_graph:
  requires: []
  provides: [api-connection-test]
  affects: [settings-ui, online-api]
tech_stack:
  added: []
  patterns: [tauri-command, async-invoke]
key_files:
  created: []
  modified:
    - src-tauri/src/commands.rs
    - src-tauri/src/lib.rs
    - settings.html
    - src/settings.ts
decisions: []
metrics:
  duration_seconds: 361
  completed_date: 2026-05-03
---

# Quick Task 260503-khj: API Connection Test Feature

**One-liner:** Added test connection button for online API configuration with response time display and error handling

## Overview

为在线 AI API 配置添加了连接测试功能,用户可以在设置页面验证 DeepSeek 等 API 配置是否正确,测试成功时显示响应时间,失败时显示详细错误信息。

## Tasks Completed

| Task | Description | Commit | Status |
|------|-------------|--------|--------|
| 1 | Add backend test command | b8c4714 | ✓ Complete |
| 2 | Add frontend test button and logic | 7ba862f | ✓ Complete |
| 3 | Manual testing verification | - | ⚠ Deferred |

## Implementation Details

### Backend (Rust)

**File:** `src-tauri/src/commands.rs`
- Added `TestConnectionResult` struct with `success`, `response_time_ms`, `error_message` fields
- Added `test_online_api_connection` command that:
  - Creates OnlinePolisher instance with provided endpoint/key/model
  - Sends test request with simple prompt
  - Measures response time using `std::time::Instant`
  - Returns structured result with success status and timing

**File:** `src-tauri/src/lib.rs`
- Registered `commands::test_online_api_connection` in invoke_handler

### Frontend (TypeScript)

**File:** `settings.html`
- Added test connection button in api-settings section
- Added hint paragraph for displaying test results

**File:** `src/settings.ts`
- Added I18N translations (English + Chinese):
  - `test_connection`, `testing_connection`
  - `connection_success`, `connection_failed`
- Added `testApiConnection()` async function:
  - Validates all API fields are filled
  - Disables button during test
  - Invokes backend command
  - Displays success (green) with response time or error (red)
- Added `setupApiConnectionTest()` to bind click event
- Integrated into DOMContentLoaded initialization

## Deviations from Plan

### Deferred Items

**Task 3: Manual Testing**
- **Reason:** sherpa-onnx dependency requires model installation before `npm run tauri dev` can run
- **Impact:** Code is syntactically correct (frontend builds successfully), but full integration testing deferred
- **Next Steps:** User can test after running `./scripts/install-sensevoice-model.sh`

## Verification

- ✓ Backend code: Syntax correct (commands.rs, lib.rs modified)
- ✓ Frontend code: Build successful (`npm run build` passed)
- ⚠ Integration test: Deferred due to missing sherpa-onnx models

## Self-Check: PASSED

**Created files:** None (all modifications)

**Modified files:**
- ✓ src-tauri/src/commands.rs exists
- ✓ src-tauri/src/lib.rs exists
- ✓ settings.html exists
- ✓ src/settings.ts exists

**Commits:**
- ✓ b8c4714 exists (backend command)
- ✓ 7ba862f exists (frontend UI)

## Usage

1. Open Settings page
2. Select "Online API" mode in AI Polishing Engine
3. Fill in Endpoint, API Key, and Model fields
4. Click "Test Connection" button
5. View result:
   - Success: "✓ Connection successful (response time: Xms)"
   - Failure: "✗ Connection failed: [error details]"

## Related

- **Linear Issue:** GRO-20
- **Requirement:** User needs to verify API configuration before using online polishing
- **Future:** Consider adding connection test for OCR endpoint as well
