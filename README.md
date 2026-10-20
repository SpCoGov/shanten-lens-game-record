# 向听镜牌谱工具

向听镜的独立牌谱插件，支持查看与编辑牌谱、导入导出 JSON、在线获取、脱敏以及替换下一次游戏内打开的牌谱。支持中文和日文，并跟随宿主主题与牌面样式。

可在向听镜插件市场安装 Windows x64 版；其他系统在 [Releases](https://github.com/SpCoGov/shanten-lens-game-record/releases) 下载对应系统的 `.plugin.zip`，在向听镜“插件”页面导入并启用，批准读取、修改和注入数据包权限。启用后，牌谱入口会出现在插件页面列表中；禁用或卸载后入口移除。

需要包含牌谱插件拆分更新的向听镜版本。现有 v3.2.0 正式版尚不包含这些接口；请使用包含拆分更新的主程序构建。旧宿主缺少插件事件、请求和页面跳转接口时会提示升级。在线获取和游戏内替换需要已连接的雀魂代理会话；本地导入、编辑与导出不需要游戏连接。

替换只作用于下一次 `.lq.Lobby.fetchGameRecord` 响应，重启或禁用插件会取消未执行的替换。插件不会修改原始 JSON 文件；请在卸载前自行导出需要保留的编辑内容。

## 构建

需要 pnpm、Rust 和 Python 3。执行：

```powershell
pnpm install --frozen-lockfile
cargo test --locked
pnpm run pack:plugin
```

安装包与 SHA-256 元数据生成到 `dist/`。发布工作流分别构建 Windows x64、macOS Intel 和 macOS Apple Silicon 安装包；推送版本标签时发布独立 GitHub Release，手动触发仅生成可下载的构建产物。发布成功后自动更新 `marketplace/update.json`，插件市场会据此校验并同步新版本；市场当前仅登记 Windows x64 包。

牌谱页面、牌面组件、目录数据与协议来自 Apache 2.0 许可的 [shanten-lens](https://github.com/SpCoGov/shanten-lens)，保留原项目许可证。`proto/liqi.proto` 和 `frontend/data/` 是本仓库构建输入，更新协议时应同步对应数据。
