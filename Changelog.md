## v2.5.3

### 🐞 修复发布

- 启用 `createUpdaterArtifacts`，Release 生成完整 `.sig` 与 `latest.json`（与上游 25 个 Assets 一致）
- WinGet 首包提交改用 `komac submit`（`V2Free.ClashVergeForV2free`）

## v2.5.2

### ✨ 品牌与发布

- 独立品牌 **Clash Verge for v2free**（产品名、Bundle ID、WinGet `V2Free.ClashVergeForV2free`）
- 配置 Tauri 更新签名，Release 补齐 `.sig` 与 `latest.json`
- macOS 构建 job 启用签名密钥上传

## v2.5.1

- **Mihomo(Meta) 内核升级至 v1.19.25**
- **Mihomo Alpha 内核升级至 alpha-0ddf4bb**
- **同步 meta-rules-dat 最新资源（geosite.dat / geoip.dat / Country.mmdb）**



### 🐞 修复问题

- 备份设置功能异常
- 修复 Windows 节点交互异常

