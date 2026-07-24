import { context, getOctokit } from '@actions/github'

import { resolveUpdateLog } from './updatelog.mjs'

const UPDATE_TAG_NAME = 'updater'
const UPDATE_JSON_FILE = 'update-fixed-webview2.json'
const UPDATE_JSON_PROXY = 'update-fixed-webview2-proxy.json'

function compareSemverTag(a, b) {
  const parse = (tagName) => {
    const match = /^v(\d+)\.(\d+)\.(\d+)$/.exec(tagName)
    if (!match) return null
    return [Number(match[1]), Number(match[2]), Number(match[3])]
  }

  const va = parse(a)
  const vb = parse(b)
  if (!va && !vb) return 0
  if (!va) return 1
  if (!vb) return -1

  for (let i = 0; i < 3; i++) {
    if (va[i] !== vb[i]) return va[i] - vb[i]
  }
  return 0
}

async function findLatestPublishableStableTag(github, options, stableTags) {
  for (const tag of stableTags) {
    try {
      const { data: release } = await github.rest.repos.getReleaseByTag({
        ...options,
        tag: tag.name,
      })

      if (release.draft) {
        console.log(`Skipping ${tag.name}: release is still a draft`)
        continue
      }

      const hasWin64 = release.assets.some((asset) =>
        asset.name.endsWith('x64_fixed_webview2-setup.exe'),
      )
      if (!hasWin64) {
        console.log(
          `Skipping ${tag.name}: missing x64_fixed_webview2-setup.exe`,
        )
        continue
      }

      return { tag, release }
    } catch (error) {
      if (error.status === 404) {
        console.log(`Skipping ${tag.name}: no GitHub release found`)
        continue
      }
      throw error
    }
  }

  return null
}

async function resolveUpdater() {
  if (process.env.GITHUB_TOKEN === undefined) {
    throw new Error('GITHUB_TOKEN is required')
  }

  const options = { owner: context.repo.owner, repo: context.repo.repo }
  const github = getOctokit(process.env.GITHUB_TOKEN)

  let allTags = []
  let page = 1
  const perPage = 100

  while (true) {
    const { data: pageTags } = await github.rest.repos.listTags({
      ...options,
      per_page: perPage,
      page,
    })

    allTags = allTags.concat(pageTags)
    if (pageTags.length < perPage) break
    page++
  }

  const stableTagRegex = /^v\d+\.\d+\.\d+$/
  const stableTags = allTags
    .filter((t) => stableTagRegex.test(t.name))
    .sort((a, b) => compareSemverTag(b.name, a.name))

  const selected = await findLatestPublishableStableTag(
    github,
    options,
    stableTags,
  )
  if (!selected) {
    console.log('No publishable fixed-webview2 release found; skipping upload')
    return
  }

  const { tag, release: latestRelease } = selected
  console.log('Selected tag for fixed-webview2 updater:', tag.name)

  const updateData = {
    name: tag.name,
    notes: await resolveUpdateLog(tag.name),
    pub_date: new Date().toISOString(),
    platforms: {
      'windows-x86_64': { signature: '', url: '' },
      'windows-aarch64': { signature: '', url: '' },
      'windows-x86': { signature: '', url: '' },
      'windows-i686': { signature: '', url: '' },
    },
  }

  const promises = latestRelease.assets.map(async (asset) => {
    const { name, browser_download_url } = asset

    if (name.endsWith('x64_fixed_webview2-setup.exe')) {
      updateData.platforms['windows-x86_64'].url = browser_download_url
    }
    if (name.endsWith('x64_fixed_webview2-setup.exe.sig')) {
      const sig = await getSignature(browser_download_url)
      updateData.platforms['windows-x86_64'].signature = sig
    }

    if (name.endsWith('x86_fixed_webview2-setup.exe')) {
      updateData.platforms['windows-x86'].url = browser_download_url
      updateData.platforms['windows-i686'].url = browser_download_url
    }
    if (name.endsWith('x86_fixed_webview2-setup.exe.sig')) {
      const sig = await getSignature(browser_download_url)
      updateData.platforms['windows-x86'].signature = sig
      updateData.platforms['windows-i686'].signature = sig
    }

    if (name.endsWith('arm64_fixed_webview2-setup.exe')) {
      updateData.platforms['windows-aarch64'].url = browser_download_url
    }
    if (name.endsWith('arm64_fixed_webview2-setup.exe.sig')) {
      const sig = await getSignature(browser_download_url)
      updateData.platforms['windows-aarch64'].signature = sig
    }
  })

  await Promise.allSettled(promises)
  console.log(updateData)

  Object.entries(updateData.platforms).forEach(([key, value]) => {
    if (!value.url) {
      console.log(`[Error]: failed to parse release for "${key}"`)
      delete updateData.platforms[key]
    }
  })

  const win64 = updateData.platforms['windows-x86_64']
  if (!win64?.url || !win64?.signature) {
    console.log(
      '[Error]: missing Windows x64 fixed-webview2 metadata, skipping upload',
    )
    return
  }

  const updateDataNew = JSON.parse(JSON.stringify(updateData))
  Object.entries(updateDataNew.platforms).forEach(([key, value]) => {
    if (value.url) {
      updateDataNew.platforms[key].url = 'https://update.hwdns.net/' + value.url
    }
  })

  const { data: updateRelease } = await github.rest.repos.getReleaseByTag({
    ...options,
    tag: UPDATE_TAG_NAME,
  })

  for (const asset of updateRelease.assets) {
    if (asset.name === UPDATE_JSON_FILE) {
      await github.rest.repos.deleteReleaseAsset({
        ...options,
        asset_id: asset.id,
      })
    }
    if (asset.name === UPDATE_JSON_PROXY) {
      await github.rest.repos
        .deleteReleaseAsset({ ...options, asset_id: asset.id })
        .catch(console.error)
    }
  }

  await github.rest.repos.uploadReleaseAsset({
    ...options,
    release_id: updateRelease.id,
    name: UPDATE_JSON_FILE,
    data: JSON.stringify(updateData, null, 2),
  })

  await github.rest.repos.uploadReleaseAsset({
    ...options,
    release_id: updateRelease.id,
    name: UPDATE_JSON_PROXY,
    data: JSON.stringify(updateDataNew, null, 2),
  })
}

async function getSignature(url) {
  const response = await fetch(url, {
    method: 'GET',
    headers: { 'Content-Type': 'application/octet-stream' },
  })

  return response.text()
}

resolveUpdater().catch(console.error)
