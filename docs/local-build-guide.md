# 本地源码打包指南（MaScribe）

这份文档用于“在自己的电脑上，从源码打包出可安装应用”。

## 1. 适用场景

- 你想基于当前源码自己打包
- 你不想等官方发布包
- 你要验证本地改动后再发布

---

## 2. 通用前置条件

- Git
- Node.js 18+
- Rust stable

先确认：

```bash
node -v
npm -v
rustc --version
cargo --version
```

---

## 3. 拉取源码

```bash
git clone git@github.com:xiaozhenliu/mascribe.git
cd mascribe
npm install
```

---

## 4. macOS 打包

### 4.1 额外依赖

- Xcode Command Line Tools

```bash
xcode-select --install
```

### 4.2 正常打包（带签名）

```bash
npx tauri build
```

### 4.3 无签名打包（本地测试可用）

如果签名证书或钥匙串授权有问题，用：

```bash
npx tauri build --no-sign
```

### 4.4 产物位置

- App：`src-tauri/target/release/bundle/macos/MaScribe.app`
- DMG：`src-tauri/target/release/bundle/dmg/MaScribe_*.dmg`

---

## 5. Windows 说明（当前状态）

当前仓库尚未发布可交付的 Windows 安装包，因此这里不提供 `MSI/EXE` 产物说明。

如果你只在 WSL 开发，通常无法直接产出可分发的 Windows 安装包；后续建议在原生 Windows 环境补齐 Node.js、Rust、Build Tools 后再做 Windows 打包验证。

---

## 6. 发布前建议检查

1. 应用名是否正确：`MaScribe`
2. 包标识是否正确：`com.mascribe`
3. 首次启动权限流程是否正常（麦克风、辅助功能等）
4. 自动粘贴是否正常
5. 录音与截图目录是否写入到 `com.mascribe`

---

## 7. 常见问题

### 7.1 macOS 签名失败：`no identity found`

- 先查看证书：

```bash
security find-identity -v -p codesigning
```

- 或先用 `--no-sign` 生成测试包。

### 7.2 打包成功但运行被系统拦截

- macOS：`系统设置 -> 隐私与安全性` 中选择 `仍要打开`
