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

Write-Host "Syncing winget-pkgs fork..."
Invoke-Komac sync-fork

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
  Invoke-Komac update $PkgId --version $Version --urls ($urls -join ' ') --submit --release-notes-url $notesUrl
}

Write-Host 'Cleaning up merged komac branches...'
Invoke-Komac cleanup --only-merged
Write-Host 'WinGet submit finished.'
