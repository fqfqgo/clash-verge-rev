# Submit or update V2Free.ClashVergeForV2free on winget-pkgs via komac.
$ErrorActionPreference = 'Stop'
# Native komac exit codes must fail the step (PS 7.3+).
if (Get-Variable -Name PSNativeCommandUseErrorActionPreference -ErrorAction SilentlyContinue) {
  $PSNativeCommandUseErrorActionPreference = $true
}

function Invoke-Komac {
  param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $KomacArgs
  )
  Write-Host ">> komac $($KomacArgs -join ' ')"
  & komac @KomacArgs
  if ($LASTEXITCODE -ne 0) {
    throw "komac failed with exit code $LASTEXITCODE : $($KomacArgs -join ' ')"
  }
}

function Sync-WingetPkgsFork {
  $forkOwner = if (-not [string]::IsNullOrWhiteSpace($env:KOMAC_FORK_OWNER)) {
    $env:KOMAC_FORK_OWNER
  } else {
    'fqfqgo'
  }
  $forkRepo = "$forkOwner/winget-pkgs"
  $headers = @{
    Authorization           = "Bearer $env:GITHUB_TOKEN"
    Accept                  = 'application/vnd.github+json'
    'User-Agent'            = 'clash-verge-rev-winget'
    'X-GitHub-Api-Version'  = '2022-11-28'
  }

  # komac sync-fork often reports UpdateRef "permissions" errors when the fork is
  # stale or the PAT lacks the workflow scope. Prefer GitHub's merge-upstream API.
  Write-Host "Syncing $forkRepo from upstream (merge-upstream)..."
  try {
    $result = Invoke-RestMethod `
      -Method Post `
      -Uri "https://api.github.com/repos/$forkRepo/merge-upstream" `
      -Headers $headers `
      -ContentType 'application/json' `
      -Body '{"branch":"master"}'
    Write-Host "Fork sync: $($result.message); merge_type=$($result.merge_type)"
  } catch {
    $detail = $_.ErrorDetails.Message
    if ([string]::IsNullOrWhiteSpace($detail)) { $detail = "$_" }
    throw @"
Failed to sync $forkRepo from microsoft/winget-pkgs.
GitHub said: $detail

Fix one of these, then re-run:
1. Open https://github.com/$forkRepo and click Sync fork -> Update branch
2. Classic PAT in WINGET_TOKEN needs scopes: public_repo (or repo) AND workflow
   (workflow is required when upstream changed files under .github/workflows)
"@
  }
}

if ([string]::IsNullOrWhiteSpace($env:GITHUB_TOKEN)) {
  throw 'GITHUB_TOKEN (WINGET_TOKEN) is required for komac to push and open a PR'
}

$PkgId = 'V2Free.ClashVergeForV2free'
$Version = $env:VERSION
if ([string]::IsNullOrWhiteSpace($Version)) {
  throw 'VERSION environment variable is required'
}

$Tag = "v$Version"
$Repo = $env:GITHUB_REPOSITORY
if ([string]::IsNullOrWhiteSpace($Repo)) {
  $Repo = 'fqfqgo/clash-verge-rev'
}

$manifestPath = "manifests/$($PkgId.ToLower()[0])/$($PkgId.Replace('.', '/'))"
$localManifest = "manifests/winget/V2Free.ClashVergeForV2free/$Version"

Sync-WingetPkgsFork

$packageExists = $false
try {
  $uri = "https://api.github.com/repos/microsoft/winget-pkgs/contents/$manifestPath"
  $headers = @{
    'User-Agent' = 'clash-verge-rev-winget'
    Accept       = 'application/vnd.github+json'
  }
  Invoke-RestMethod -Uri $uri -Headers $headers -Method Get | Out-Null
  $packageExists = $true
  Write-Host "Package already exists in winget-pkgs: $PkgId"
} catch {
  Write-Host "Package not found in winget-pkgs yet: $PkgId"
}

if (-not $packageExists) {
  if (-not (Test-Path $localManifest)) {
    throw "Missing local manifest directory: $localManifest"
  }
  Write-Host "Submitting initial manifest from $localManifest ..."
  Invoke-Komac submit $localManifest -y
} else {
  Write-Host "Updating package $PkgId $Version ..."
  $headers = @{
    Authorization = "Bearer $env:GITHUB_TOKEN"
    Accept        = 'application/vnd.github+json'
  }
  $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$Tag" -Headers $headers
  $urls = @(
    $release.assets |
      Where-Object { $_.name -match '_(arm64|x64|x86)-setup\.exe$' -and $_.name -notmatch 'fixed_webview2' } |
      ForEach-Object { $_.browser_download_url }
  )
  if ($urls.Count -eq 0) {
    throw "No installer URLs matched for tag $Tag"
  }
  Write-Host "Installer URLs:`n$($urls -join "`n")"
  $notesUrl = "https://github.com/$Repo/releases/tag/$Tag"
  # Pass each URL as its own argv; a joined string becomes one URL with %20 and 404s.
  $updateArgs = @('update', $PkgId, '--version', $Version, '--urls') + $urls + @('--submit', '--release-notes-url', $notesUrl)
  Invoke-Komac @updateArgs
}

Write-Host 'Cleaning up merged komac branches...'
Invoke-Komac cleanup --only-merged
Write-Host 'WinGet submit finished.'
