# =============================================================================
#  ferris-search — one-click installer (Windows / PowerShell)
#  Repo: https://github.com/lispking/ferris-search
#
#  Usage (from repo root):
#    .\install.ps1
#
#  One-liner:
#    irm https://raw.githubusercontent.com/lispking/ferris-search/main/install.ps1 | iex
# =============================================================================

$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
function Write-Info    { param($msg) Write-Host "[INFO]  $msg" -ForegroundColor Cyan }
function Write-Ok      { param($msg) Write-Host "[OK]    $msg" -ForegroundColor Green }
function Write-Warn    { param($msg) Write-Host "[WARN]  $msg" -ForegroundColor Yellow }
function Write-Err     { param($msg) Write-Host "[ERROR] $msg" -ForegroundColor Red }
function Write-Header  { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Die           { param($msg) Write-Err $msg; exit 1 }

# ---------------------------------------------------------------------------
# Detect OS / PowerShell version
# ---------------------------------------------------------------------------
Write-Header 'Detecting Environment'
if ($IsLinux -or $IsMacOS) {
    Write-Warn 'This script targets Windows. On Linux/macOS, use install.sh instead.'
    Write-Warn 'Continuing anyway (pwsh cross-platform mode)...'
}
Write-Ok "PowerShell $($PSVersionTable.PSVersion)"

# ---------------------------------------------------------------------------
# Locate repo root
# When piped via irm|iex, $PSScriptRoot is empty — clone the repo.
# When run from a local clone, $PSScriptRoot is the script directory.
# ---------------------------------------------------------------------------
Write-Header 'Locating Repository'

$CleanupDir = $null

if ($PSScriptRoot -and (Test-Path (Join-Path $PSScriptRoot 'Cargo.toml'))) {
    $RepoDir = $PSScriptRoot
    Write-Info "Using local repo at: $RepoDir"
} else {
    Write-Info 'Running via irm|iex — cloning repository...'
    $RepoDir = Join-Path $env:TEMP ("ferris-search-install-" + [System.IO.Path]::GetRandomFileName())
    git clone --depth=1 https://github.com/lispking/ferris-search.git $RepoDir
    $CleanupDir = $RepoDir
    Write-Ok "Cloned to: $RepoDir"
}

Set-Location $RepoDir

# ---------------------------------------------------------------------------
# Check Rust / cargo
# ---------------------------------------------------------------------------
Write-Header 'Checking Rust Toolchain'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Err 'cargo not found. Please install Rust first:'
    Write-Host ''
    Write-Host '    https://www.rust-lang.org/tools/install' -ForegroundColor Yellow
    Write-Host '    (download and run rustup-init.exe)'
    Write-Host ''
    Die 'Rust is required to build ferris-search.'
}
$CargoVersion = cargo --version
Write-Ok "Found $CargoVersion"

# ---------------------------------------------------------------------------
# Build and install
# ---------------------------------------------------------------------------
Write-Header 'Building & Installing ferris-search'
Write-Info 'Running: cargo install --path . --locked'
cargo install --path . --locked

$BinaryPath = Join-Path $env:USERPROFILE '.cargo\bin\ferris-search.exe'
if (Get-Command ferris-search -ErrorAction SilentlyContinue) {
    $BinaryPath = (Get-Command ferris-search).Source
}
Write-Ok "Binary installed to: $BinaryPath"

# ---------------------------------------------------------------------------
# Claude Code integration
# ---------------------------------------------------------------------------
Write-Header 'Claude Code Integration'

$ClaudeFound = $null -ne (Get-Command claude -ErrorAction SilentlyContinue)

if ($ClaudeFound) {
    Write-Info 'Claude Code CLI found. Registering MCP server...'
    claude mcp add -s user ferris-search $BinaryPath
    Write-Ok "MCP server registered: ferris-search -> $BinaryPath"

    Write-Info 'Installing Claude Code skills...'
    $SkillsTarget = Join-Path $env:USERPROFILE '.claude\skills'
    if (-not (Test-Path $SkillsTarget)) {
        New-Item -ItemType Directory -Path $SkillsTarget -Force | Out-Null
    }

    foreach ($skill in @('ferris-search-setup', 'ferris-search-tools')) {
        $Src = Join-Path $RepoDir "skills\$skill"
        if (Test-Path $Src) {
            Copy-Item -Recurse -Force $Src $SkillsTarget
            Write-Ok "Installed skill: $skill -> $SkillsTarget\$skill"
        } else {
            Write-Warn "Skill directory not found: $Src (skipping)"
        }
    }
} else {
    Write-Warn 'Claude Code CLI (claude) not found in PATH.'
    Write-Warn 'To register the MCP server after installing Claude Code, run:'
    Write-Host ''
    Write-Host "    claude mcp add -s user ferris-search `"$BinaryPath`"" -ForegroundColor Yellow
    Write-Host ''
}

# ---------------------------------------------------------------------------
# Cleanup temp clone
# ---------------------------------------------------------------------------
if ($CleanupDir -and (Test-Path $CleanupDir)) {
    Remove-Item -Recurse -Force $CleanupDir
}

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
Write-Header 'Installation Complete'
Write-Host ''
Write-Host '  ferris-search is ready!' -ForegroundColor Green
Write-Host ''
Write-Host "  Binary:   $BinaryPath"
Write-Host ''
Write-Host '  Quick test:'
Write-Host '    ferris-search --help' -ForegroundColor Cyan
Write-Host ''
Write-Host '  Claude Code (MCP):' -ForegroundColor Cyan
if ($ClaudeFound) {
    Write-Host '    Registered as a user-scoped MCP server.'
    Write-Host '    Open Claude Code and try: web_search'
} else {
    Write-Host '    Run after installing Claude Code:' -ForegroundColor Yellow
    Write-Host "      claude mcp add -s user ferris-search `"$BinaryPath`"" -ForegroundColor Yellow
}
Write-Host ''
Write-Host '  Optional env vars:'
Write-Host '    DEFAULT_SEARCH_ENGINE=bing'
Write-Host '    ALLOWED_SEARCH_ENGINES=bing,duckduckgo,brave'
Write-Host '    USE_PROXY=true  /  PROXY_URL=http://127.0.0.1:7890'
Write-Host '    BRAVE_API_KEY / EXA_API_KEY / FIRECRAWL_API_KEY / JINA_API_KEY / TAVILY_API_KEY'
Write-Host ''
Write-Host '  Docs: https://github.com/lispking/ferris-search'
Write-Host ''
