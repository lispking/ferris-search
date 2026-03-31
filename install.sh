#!/usr/bin/env bash
# =============================================================================
#  ferris-search — one-click installer
#  Repo: https://github.com/lispking/ferris-search
#
#  Usage (from repo root):
#    bash install.sh
#
#  One-liner (clones then installs):
#    curl -fsSL https://raw.githubusercontent.com/lispking/ferris-search/main/install.sh | bash
# =============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# Colors
# ---------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()   { echo -e "${RED}[ERROR]${RESET} $*" >&2; }
header()  { echo -e "\n${BOLD}${CYAN}=== $* ===${RESET}"; }
die()     { error "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Detect OS
# ---------------------------------------------------------------------------
header "Detecting OS"
case "${OSTYPE:-}" in
  linux*)   OS="linux"  ;;
  darwin*)  OS="macos"  ;;
  *)        die "Unsupported OS: ${OSTYPE:-unknown}. Use install.ps1 on Windows." ;;
esac
success "Detected OS: $OS"

# ---------------------------------------------------------------------------
# Locate repo root
# When piped via curl|bash, BASH_SOURCE[0] is empty — clone the repo.
# When run from a local clone, use the script's own directory.
# ---------------------------------------------------------------------------
header "Locating Repository"

SCRIPT_DIR=""
if [[ -n "${BASH_SOURCE[0]:-}" && "${BASH_SOURCE[0]}" != "bash" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fi

CLEANUP_DIR=""
if [[ -n "$SCRIPT_DIR" && -f "$SCRIPT_DIR/Cargo.toml" ]]; then
  REPO_DIR="$SCRIPT_DIR"
  info "Using local repo at: $REPO_DIR"
else
  info "Running via curl|bash — cloning repository..."
  REPO_DIR="$(mktemp -d)"
  CLEANUP_DIR="$REPO_DIR"
  git clone --depth=1 https://github.com/lispking/ferris-search.git "$REPO_DIR"
  success "Cloned to: $REPO_DIR"
fi

cd "$REPO_DIR"

# Cleanup temp dir on exit (only if we cloned)
if [[ -n "$CLEANUP_DIR" ]]; then
  trap 'rm -rf "$CLEANUP_DIR"' EXIT
fi

# ---------------------------------------------------------------------------
# Check Rust / cargo
# ---------------------------------------------------------------------------
header "Checking Rust Toolchain"
if ! command -v cargo &>/dev/null; then
  error "cargo not found. Please install Rust first:"
  echo
  echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo
  die "Rust is required to build ferris-search."
fi
CARGO_VERSION="$(cargo --version)"
success "Found $CARGO_VERSION"

# ---------------------------------------------------------------------------
# Build and install
# ---------------------------------------------------------------------------
header "Building & Installing ferris-search"
info "Running: cargo install --path . --locked"
cargo install --path . --locked
success "Binary installed to: $(command -v ferris-search 2>/dev/null || echo '~/.cargo/bin/ferris-search')"

BINARY_PATH="$(command -v ferris-search 2>/dev/null || echo "$HOME/.cargo/bin/ferris-search")"

# ---------------------------------------------------------------------------
# Claude Code integration
# ---------------------------------------------------------------------------
header "Claude Code Integration"

if command -v claude &>/dev/null; then
  info "Claude Code CLI found. Registering MCP server..."
  claude mcp add -s user ferris-search "$BINARY_PATH"
  success "MCP server registered: ferris-search → $BINARY_PATH"

  info "Installing Claude Code skills..."
  SKILLS_TARGET="$HOME/.claude/skills"
  mkdir -p "$SKILLS_TARGET"

  for skill in ferris-search-setup ferris-search-tools; do
    SRC="$REPO_DIR/skills/$skill"
    if [[ -d "$SRC" ]]; then
      cp -r "$SRC" "$SKILLS_TARGET/"
      success "Installed skill: $skill → $SKILLS_TARGET/$skill"
    else
      warn "Skill directory not found: $SRC (skipping)"
    fi
  done
else
  warn "Claude Code CLI (claude) not found in PATH."
  warn "To register the MCP server after installing Claude Code, run:"
  echo
  echo "    claude mcp add -s user ferris-search \"$BINARY_PATH\""
  echo
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
header "Installation Complete"
echo
echo -e "  ${GREEN}${BOLD}ferris-search is ready!${RESET}"
echo
echo -e "  Binary:   ${BOLD}$BINARY_PATH${RESET}"
echo
echo -e "  Quick test:"
echo -e "    ${CYAN}ferris-search --help${RESET}"
echo
echo -e "  ${CYAN}Claude Code (MCP):${RESET}"
if command -v claude &>/dev/null; then
  echo -e "    Registered as a user-scoped MCP server."
  echo -e "    Open Claude Code and try: web_search"
else
  echo -e "    ${YELLOW}Run after installing Claude Code:${RESET}"
  echo -e "      ${YELLOW}claude mcp add -s user ferris-search \"$BINARY_PATH\"${RESET}"
fi
echo
echo -e "  ${CYAN}Optional env vars:${RESET}"
echo -e "    DEFAULT_SEARCH_ENGINE=bing"
echo -e "    ALLOWED_SEARCH_ENGINES=bing,duckduckgo,brave"
echo -e "    USE_PROXY=true  /  PROXY_URL=http://127.0.0.1:7890"
echo -e "    BRAVE_API_KEY / EXA_API_KEY / FIRECRAWL_API_KEY / JINA_API_KEY / TAVILY_API_KEY"
echo
echo -e "  Docs: https://github.com/lispking/ferris-search"
echo
