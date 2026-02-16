# MaScribe（马上听写）

MaScribe 是一个本地语音输入工具（macOS / Windows）。
按下快捷键说话，停止后文本会自动粘贴到当前光标位置。

- 默认快捷键：`Alt+Space`
- 支持中英混合等多语言转写
- 支持 AI 润色（本地模型或在线 API）
- 支持截图 OCR 上下文（用于同音字纠错）

## 下载与安装

优先使用发布包（推荐）：

- 打开仓库 **Releases** 下载最新版
- macOS：下载 `.dmg`，拖拽到 `Applications`
- Windows：下载 `.msi` 或 `.exe`

## 快速开始

1. 启动 MaScribe（菜单栏/系统托盘图标）
2. 首次按提示授予权限（尤其是 macOS 的辅助功能、麦克风）
3. 按 `Alt+Space` 开始录音，再按一次结束
4. 文本将自动粘贴到当前输入位置

## 常见问题

### 1) 自动粘贴失败（macOS）

通常是辅助功能权限未生效：

1. 打开 `系统设置 -> 隐私与安全性 -> 辅助功能`
2. 确认 `MaScribe.app` 已开启权限（必要时删掉后重新添加）
3. 重启 MaScribe 再试

可选重置命令：

```bash
tccutil reset Accessibility com.mascribe
```

### 2) 换了新版本后配置不见了

品牌从旧版迁移到 `MaScribe` 后，配置目录为：

- 新版：`~/Library/Application Support/com.mascribe/`
- 旧版：`~/Library/Application Support/com.mac-voice-input/`

如果你有旧数据，先迁移再删除旧目录。

## 配置与文档

详细文档请看 `docs/`：

- 在线 API 配置：`docs/online-api-guide-zh.md`
- Windows 构建：`docs/windows-build-guide.md`
- 产品与设计说明：`docs/PRD.md`

## 从源码构建（开发者）

```bash
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npx tauri build
```

构建产物：

- macOS App：`src-tauri/target/release/bundle/macos/MaScribe.app`
- macOS DMG：`src-tauri/target/release/bundle/dmg/MaScribe_*.dmg`
- Windows 安装包：`src-tauri/target/release/bundle/msi/` 或 `nsis/`

## License

MIT
