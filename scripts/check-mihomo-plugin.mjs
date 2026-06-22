import fs from 'node:fs'
import path from 'node:path'

// 约束：mihomo 通信插件的「Rust 端」与「前端 JS 端」都必须钉死 commit（可复现构建）。
// 两半各自配套不同的对象，commit 允许不同：
//   - Rust 端（Cargo.toml rev）做内核响应的反序列化，须与「浮动版」内核的数据格式兼容；
//     历史上某版只接受大写配置枚举（INFO/Strict），而内核返回小写（info/strict），
//     导致首页 "Core communication error"、Clash Info 空白、代理/规则页空白。
//   - 前端端（package.json #commit）只是 invoke 包装 + TS 类型，须与本仓前端代码配套。
// 升级任一半都要分别验证：Rust 端验通信、前端端验 typecheck/页面，确认后再钉死。

const cwd = process.cwd()
const errors = []

const cargoToml = fs.readFileSync(
  path.join(cwd, 'src-tauri/Cargo.toml'),
  'utf-8',
)
const rustLine = cargoToml
  .split('\n')
  .find((line) => line.trimStart().startsWith('tauri-plugin-mihomo ='))

const rustRev = rustLine?.match(/\brev\s*=\s*"([0-9a-f]{7,40})"/)?.[1]

if (!rustLine) {
  errors.push('src-tauri/Cargo.toml: 未找到 tauri-plugin-mihomo 依赖')
} else if (!rustRev) {
  errors.push(
    `src-tauri/Cargo.toml: tauri-plugin-mihomo 未钉 rev：\n      ${rustLine.trim()}`,
  )
}

const pkg = JSON.parse(fs.readFileSync(path.join(cwd, 'package.json'), 'utf-8'))
const jsSpec = pkg.dependencies?.['tauri-plugin-mihomo-api']
const jsRev = jsSpec?.match(/#([0-9a-f]{7,40})$/)?.[1]

if (!jsSpec) {
  errors.push('package.json: 未找到 tauri-plugin-mihomo-api 依赖')
} else if (!jsRev) {
  errors.push(
    `package.json: tauri-plugin-mihomo-api 未钉 #commit：\n      "tauri-plugin-mihomo-api": "${jsSpec}"`,
  )
}

if (errors.length) {
  console.error('❌ mihomo 插件未钉死 commit（须可复现构建）：\n')
  for (const e of errors) console.error('  - ' + e)
  console.error(
    '\nRust 端（Cargo.toml rev）与前端端（package.json #commit）都必须钉死 commit；升级任一半须分别验证后再钉。',
  )
  process.exit(1)
}

console.log(`✅ mihomo 插件两半均已钉死：Rust rev=${rustRev}，前端 #${jsRev}`)
