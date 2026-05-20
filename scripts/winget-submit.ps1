# Submit or update V2Free.ClashVergeForV2free on winget-pkgs via komac.
$ErrorActionPreference = 'Stop'

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
komac sync-fork

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
  komac submit $localManifest -y
} else {
  Write-Host "Updating package $PkgId $Version ..."
  $headers = @{
    Authorization = "Bearer $env:GITHUB_TOKEN"
    Accept        = 'application/vnd.github+json'
  }
  $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$Tag" -Headers $headers
  $urls = (
    $release.assets |
      Where-Object { $_.name -match '_(arm64|x64|x86)-setup\.exe$' -and $_.name -notmatch 'fixed_webview2' }
  ).browser_download_url -join ' '
  if ([string]::IsNullOrWhiteSpace($urls)) {
    throw "No installer URLs matched for tag $Tag"
  }
  $notesUrl = "https://github.com/$Repo/releases/tag/$Tag"
  komac update $PkgId --version $Version --urls $urls --submit --release-notes-url $notesUrl
}

Write-Host 'Cleaning up merged komac branches...'
komac cleanup --only-merged
