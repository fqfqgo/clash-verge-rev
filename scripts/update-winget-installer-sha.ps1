# Refresh InstallerSha256 in manifests/winget for a release version.
param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
  [string]$Repo = 'fqfqgo/clash-verge-rev'
)

$ErrorActionPreference = 'Stop'
$Tag = "v$Version"
$manifestFile = "manifests/winget/V2Free.ClashVergeForV2free/$Version/V2Free.ClashVergeForV2free.installer.yaml"

if (-not (Test-Path $manifestFile)) {
  throw "Manifest not found: $manifestFile"
}

$assets = @(
  @{ Arch = 'x64'; Pattern = "Clash.Verge_${Version}_x64-setup.exe" },
  @{ Arch = 'arm64'; Pattern = "Clash.Verge_${Version}_arm64-setup.exe" }
)

$content = Get-Content $manifestFile -Raw
foreach ($asset in $assets) {
  $url = "https://github.com/$Repo/releases/download/$Tag/$($asset.Pattern)"
  $temp = Join-Path $env:TEMP $asset.Pattern
  Write-Host "Downloading $url ..."
  Invoke-WebRequest -Uri $url -OutFile $temp -UseBasicParsing
  $hash = (Get-FileHash -Path $temp -Algorithm SHA256).Hash
  Write-Host "$($asset.Arch): $hash"
  $content = $content -replace `
    "(?s)(- Architecture: $($asset.Arch)\r?\n(?:.*\r?\n)*?    InstallerSha256: )[0-9A-F]+", `
    "`${1}$hash"
}

Set-Content -Path $manifestFile -Value $content.TrimEnd() + "`n" -NoNewline
Write-Host "Updated $manifestFile"
