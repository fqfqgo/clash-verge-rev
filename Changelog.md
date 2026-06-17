## v2.5.9

### 🐞 修复问题

- **macOS**：从统一包名 `Clash Verge.app` 启动时，静默将旧版 `Clash Verge for v2free.app` 移入废纸篓，避免双份应用并存

## v2.5.8

### 🐞 修复问题

- **Windows 应用内更新**：安装覆盖前先优雅停止 `clash_verge_service`，避免 verge-mihomo 子进程占用文件导致替换失败；覆盖前额外等待句柄释放

## v2.5.7

### 🐞 修复问题

- **Windows 应用内更新**：安装前释放内核进程、允许立即退出，安装包内结束主进程并覆盖 `clash-verge.exe`，由安装器 `/R` 拉起新版本（不再误调 `relaunch()` 回到旧程序）
- **安装路径**：统一为 `Program Files\Clash Verge`；检测到旧版 fork 目录时先静默卸载再安装（保留用户配置）
- **更新源**：`update.json` 仅在对应 Release 已发布且含 x64 安装包时才写入，避免指向 404

## v2.5.6

### 🐞 修复问题

- 代理页：虚拟列表异常时降级为普通列表渲染，避免页面空白
- 代理页：进入页面及 profile 切换时主动刷新代理数据
- 修复代理列表渲染缓存未随内核数据更新而清空的问题

## v2.5.5

### 🐞 修复问题

- 修复代理页虚拟列表在滚动容器未就绪或滚动位置异常时显示空白的问题

## v2.5.4

### ✨ 品牌与界面定制

- 恢复 fork 文档 / GitHub 链接与 Home 页「启动浏览器」按钮
- 窗口标题、设置页与 Home 系统信息统一显示 fork 版本标签

## v2.5.3

### 🐞 修复发布

- 启用 `createUpdaterArtifacts`，Release 生成完整 `.sig` 与 `latest.json`（与上游 25 个 Assets 一致）
- WinGet 首包提交改用 `komac submit`（`V2Free.ClashVergeForV2free`）
- Release 安装包文件名恢复为上游格式 `Clash.Verge_*`，保证旧版应用内更新可用

## v2.5.2

### ✨ 品牌与发布

- 独立品牌定制（产品名、Bundle ID、WinGet `V2Free.ClashVergeForV2free`）
- 配置 Tauri 更新签名，Release 补齐 `.sig` 与 `latest.json`
- macOS 构建 job 启用签名密钥上传

## v2.5.1

- **Mihomo(Meta) 内核升级至 v1.19.25**
- **Mihomo Alpha 内核升级至 alpha-0ddf4bb**
- **同步 meta-rules-dat 最新资源（geosite.dat / geoip.dat / Country.mmdb）**



### 🐞 修复问题

- 备份设置功能异常
- 修复 Windows 节点交互异常

