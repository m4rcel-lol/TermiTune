#!/usr/bin/env bash
# TermiTune Build Script
# Arch Linux native build

set -euo pipefail

BINARY_NAME="termitune"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="${HOME}/.config/termitune"
THEMES_DIR="${CONFIG_DIR}/themes"

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

banner() {
    echo -e "${CYAN}"
    echo "  ████████╗███████╗██████╗ ███╗   ███╗██╗████████╗██╗   ██╗███╗   ██╗███████╗"
    echo "     ██╔══╝██╔════╝██╔══██╗████╗ ████║██║╚══██╔══╝██║   ██║████╗  ██║██╔════╝"
    echo "     ██║   █████╗  ██████╔╝██╔████╔██║██║   ██║   ██║   ██║██╔██╗ ██║█████╗  "
    echo "     ██║   ██╔══╝  ██╔══██╗██║╚██╔╝██║██║   ██║   ██║   ██║██║╚██╗██║██╔══╝  "
    echo "     ██║   ███████╗██║  ██║██║ ╚═╝ ██║██║   ██║   ╚██████╔╝██║ ╚████║███████╗"
    echo "     ╚═╝   ╚══════╝╚═╝  ╚═╝╚═╝     ╚═╝╚═╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝╚══════╝"
    echo -e "${RESET}"
    echo -e "${BOLD}  TUI MP3 Music Player — v0.1.0 — Build Script${RESET}"
    echo
}

info()    { echo -e "  ${CYAN}[→]${RESET} $1"; }
success() { echo -e "  ${GREEN}[✓]${RESET} $1"; }
warn()    { echo -e "  ${YELLOW}[!]${RESET} $1"; }
error()   { echo -e "  ${RED}[✗]${RESET} $1"; exit 1; }

# ─── Dependency checks ────────────────────────────────────────────────────────

check_deps() {
    info "Checking dependencies..."

    # Rust toolchain
    if ! command -v cargo &>/dev/null; then
        error "cargo not found. Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi
    success "cargo $(cargo --version 2>/dev/null | awk '{print $2}')"

    # Arch packages
    local PKGS=("alsa-lib" "pkgconf")
    for pkg in "${PKGS[@]}"; do
        if pacman -Q "$pkg" &>/dev/null; then
            success "$pkg"
        else
            warn "$pkg not found — installing..."
            sudo pacman -S --noconfirm "$pkg" || error "Failed to install $pkg"
        fi
    done

    # Optional: libpulse
    if pacman -Q "libpulse" &>/dev/null; then
        success "libpulse (optional)"
    else
        warn "libpulse not found (optional, will use ALSA)"
    fi
}

# ─── Build ────────────────────────────────────────────────────────────────────

build() {
    local profile="${1:-release}"
    info "Building TermiTune (${profile})..."

    if [[ "$profile" == "release" ]]; then
        cargo build --release 2>&1 | tail -5
        BINARY_PATH="target/release/${BINARY_NAME}"
    else
        cargo build 2>&1 | tail -5
        BINARY_PATH="target/debug/${BINARY_NAME}"
    fi

    if [[ ! -f "$BINARY_PATH" ]]; then
        error "Build failed — binary not found at ${BINARY_PATH}"
    fi
    success "Build complete → ${BINARY_PATH} ($(du -h "$BINARY_PATH" | cut -f1))"
}

# ─── Install ──────────────────────────────────────────────────────────────────

install_binary() {
    info "Installing to ${INSTALL_DIR}..."
    if [[ -w "$INSTALL_DIR" ]]; then
        cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/"
    else
        sudo cp "target/release/${BINARY_NAME}" "${INSTALL_DIR}/"
    fi
    success "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
}

setup_config() {
    info "Setting up config directories..."
    mkdir -p "${CONFIG_DIR}" "${THEMES_DIR}"
    mkdir -p "${HOME}/.config/termitune/playlists"
    success "Config dirs ready at ${CONFIG_DIR}"
}

# ─── Main ─────────────────────────────────────────────────────────────────────

banner

case "${1:-build}" in
    check)
        check_deps
        ;;
    debug)
        check_deps
        build debug
        setup_config
        info "Run with: ./target/debug/${BINARY_NAME}"
        ;;
    build)
        check_deps
        build release
        setup_config
        echo
        echo -e "  ${BOLD}${GREEN}✓ TermiTune built successfully!${RESET}"
        echo -e "  Run: ${CYAN}./target/release/${BINARY_NAME}${RESET}"
        echo -e "  Or:  ${CYAN}$0 install${RESET} to install system-wide"
        ;;
    install)
        check_deps
        build release
        install_binary
        setup_config
        echo
        echo -e "  ${BOLD}${GREEN}✓ TermiTune installed!${RESET}"
        echo -e "  Run: ${CYAN}termitune${RESET}"
        ;;
    uninstall)
        info "Removing binary..."
        sudo rm -f "${INSTALL_DIR}/${BINARY_NAME}"
        success "Uninstalled"
        ;;
    clean)
        cargo clean
        success "Cleaned build artifacts"
        ;;
    *)
        echo "Usage: $0 [check|debug|build|install|uninstall|clean]"
        exit 1
        ;;
esac
