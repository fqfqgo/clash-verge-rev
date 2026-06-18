import fs from 'node:fs'
import path from 'node:path'

// 约束：mihomo 通信插件必须跟随上游默认分支，不得单独冻结。
// 原因：内核 verge-mihomo 在 prebuild.mjs 中按 releases/latest 构建时拉取（不锁版本）。
// 若插件被钉死在旧 branch/rev/tag，新内核与旧插件通信协议错配，会导致代理页空白。
// 插件（Rust + 前端 API）须与内核成套升级。

const cwd = process.cwd()
const errors = []

const cargoToml = fs.readFileSync(
  path.join(cwd, 'src-tauri/Cargo.toml'),
  'utf-8',
)
const rustLine = cargoToml
  .split('\n')
  .find((line) => line.trimStart().startsWith('tauri-plugin-mihomo ='))

if (!rustLine) {
  errors.push('src-tauri/Cargo.toml: 未找到 tauri-plugin-mihomo 依赖')
} else if (/\b(branch|rev|tag)\s*=/.test(rustLine)) {
  errors.push(`src-tauri/Cargo.toml 插件被钉死：\n      ${rustLine.trim()}`)
}

const pkg = JSON.parse(fs.readFileSync(path.join(cwd, 'package.json'), 'utf-8'))
const jsSpec = pkg.dependencies?.['tauri-plugin-mihomo-api']

if (!jsSpec) {
  errors.push('package.json: 未找到 tauri-plugin-mihomo-api 依赖')
} else if (jsSpec.includes('#')) {
  errors.push(
    `package.json 前端插件被钉死：\n      "tauri-plugin-mihomo-api": "${jsSpec}"`,
  )
}

if (errors.length) {
  console.error('❌ mihomo 插件被单独冻结，违反「内核 + 插件成套升级」约束：\n')
  for (const e of errors) console.error('  - ' + e)
  console.error(
    '\n内核 verge-mihomo 构建时拉取 releases/latest（不锁版本），插件钉死在旧 ref 会与新内核通信错配，导致代理页空白。',
  )
  console.error(
    '请去掉 branch/rev/tag 与 #ref，让插件跟随默认分支、与内核成套升级。',
  )
  process.exit(1)
}

console.log('✅ mihomo 插件跟随上游默认分支，未被单独冻结')
