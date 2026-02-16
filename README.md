# MaScribe（马上听写）

MaScribe 是一个本地语音输入工具（当前优先支持 macOS）。
按下快捷键说话，停止后文本会自动粘贴到当前光标位置。

- 默认快捷键：`Alt+Space`
- 支持中英混合等多语言转写
- 支持 AI 润色（本地模型或在线 API）
- 支持截图 OCR 上下文（用于同音字纠错）

## 重要：首次使用前必须准备本地转写模型

当前版本不会自动下载语音模型。  
如果本地缺少模型文件，应用将无法完成语音转写。

### 安装步骤（macOS）

方式 A（你是源码用户，在仓库根目录执行）：

```bash
./scripts/install-sensevoice-model.sh
```

方式 B（你只下载了 DMG，没有源码）：

```bash
mkdir -p ~/.openclaw/models/sensevoice
cd ~/.openclaw/models/sensevoice
curl -L -o sensevoice.tar.bz2 \
  https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17.tar.bz2
tar -xjf sensevoice.tar.bz2
rm -f sensevoice.tar.bz2
```

默认模型目录（macOS）：

`~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/`

安装成功后目录内至少有：

- `model.int8.onnx`
- `tokens.txt`

可用下面命令验证：

```bash
ls ~/.openclaw/models/sensevoice/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2024-07-17/{model.int8.onnx,tokens.txt}
```

## 下载与安装

优先使用发布包（推荐）：

- 打开仓库 **Releases** 下载最新版
- macOS：下载 `.dmg`，拖拽到 `Applications`

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

## 配置与文档

详细文档请看 `docs/`：

- 本地源码打包：`docs/local-build-guide.md`
- macOS 安装与权限：`docs/macos-guide.md`
- 在线 API 配置：`docs/online-api-guide-zh.md`
- 产品与设计说明：`docs/PRD.md`

## 从源码构建（开发者）

优先参考：`docs/local-build-guide.md`

```bash
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
npx tauri build
```

构建产物：

- macOS App：`src-tauri/target/release/bundle/macos/MaScribe.app`
- macOS DMG：`src-tauri/target/release/bundle/dmg/MaScribe_*.dmg`

## License

MIT
