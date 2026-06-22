# Zoho Desktop Widget — local cross-platform build & release
#
# Tauri cannot cross-compile between OSes: a Windows installer must be built on
# Windows, a .app on macOS, an AppImage on Linux. So the release flow is:
#
#   1. On EACH machine (linux / windows / macos), on the LAN (so npm can reach
#      the @gio bigsu registry):
#         make release VERSION=v0.1.0
#      -> builds, signs, creates/updates a DRAFT GitHub release, uploads its
#         own platform bundle + .sig.
#   2. On ANY one machine, after all three have uploaded:
#         make latest-json VERSION=v0.1.0   # assembles updater manifest
#         make publish     VERSION=v0.1.0   # un-drafts -> becomes "latest"
#
# The updater endpoint in src-tauri/tauri.conf.json points at the public
# /releases/latest/download/latest.json, so the repo stays public and
# auto-update keeps working.
#
# Windows: run under Git Bash (recipes use bash). Install make via
#   choco install make   or   scoop install make
#
# Required env for any build that signs updater artifacts:
#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD=<password for src-tauri/keys/update.key>

SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
.ONESHELL:

# Recipes are bash. On Windows, `SHELL := bash` only resolves if Git Bash is on
# PATH; from cmd/PowerShell without it, make falls back to cmd.exe and every
# `test`/`uname`/`{ ... }` line errors cryptically. Fail fast with a clear note.
ifeq ($(OS),Windows_NT)
  ifeq (,$(shell bash -c "command -v uname"))
    $(error Run this Makefile from Git Bash, not cmd/PowerShell — see the header comment)
  endif
endif
.DEFAULT_GOAL := help

# ---- config (override on the CLI: make release VERSION=v1.2.3) ----
REPO     ?= rendyuwu/zoho-desktop-widget
VERSION  ?= v$(shell node -p "require('./package.json').version")
SIGN_KEY ?= $(CURDIR)/src-tauri/keys/update.key

# Tauri reads a file path or the raw key from this var.
export TAURI_SIGNING_PRIVATE_KEY := $(SIGN_KEY)

# Auto-load gitignored local build secrets if present:
#   ZOHO_WS_URL=wss://...                      (baked into binary)
#   LDAP_SERVER_URI=ldap://host:389            (baked into binary)
#   LDAP_BIND_TEMPLATE={user}@biznetgio.com    (baked into binary)
#   LDAP_ALLOW_INSECURE=true                   (allow cleartext ldap://)
#   TAURI_SIGNING_PRIVATE_KEY_PASSWORD=        (empty for a passwordless key)
# In CI, set these as GitHub Actions repository secrets and export them in the
# build step (same names). The LDAP server lives on the VPN, so a public
# download cannot authenticate without VPN access.
# Anything exported in the shell still wins.
-include .env.local
export ZOHO_WS_URL
export LDAP_SERVER_URI
export LDAP_BIND_TEMPLATE
export LDAP_ALLOW_INSECURE
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD

# ---- host OS detection ----
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
  HOST := linux
else ifeq ($(UNAME_S),Darwin)
  HOST := macos
else
  HOST := windows   # MINGW*/MSYS* under Git Bash
endif

TAURI := npm run tauri --

.PHONY: help
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
	  | awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
	echo ""
	echo "  Host detected: $(HOST)    VERSION: $(VERSION)"

.PHONY: check-key
check-key: ## Verify signing key + password are present
	@test -f "$(SIGN_KEY)" || { echo "missing signing key: $(SIGN_KEY)"; exit 1; }
	test -n "$${TAURI_SIGNING_PRIVATE_KEY_PASSWORD+set}" \
	  || { echo "TAURI_SIGNING_PRIVATE_KEY_PASSWORD must be defined (use empty string for a passwordless key)"; exit 1; }

.PHONY: install
install: ## npm ci (must be on LAN to reach @gio bigsu registry)
	npm ci

# ---- builds (run on the matching host) ----
.PHONY: build
build: build-$(HOST) ## Build for the current host OS

.PHONY: build-linux
build-linux: check-key ## Build Linux bundles (.AppImage/.deb)
	$(TAURI) build

.PHONY: build-windows
build-windows: check-key ## Build Windows installers (NSIS .exe/.msi)
	$(TAURI) build

.PHONY: build-mac
build-mac: check-key ## Build macOS universal (arm64 + x86_64)
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	$(TAURI) build --target universal-apple-darwin

# ---- release: build + upload this host's artifacts to a draft release ----
.PHONY: release
release: build ## Build host artifacts, then create/append draft GitHub release
	@echo ">> ensuring draft release $(VERSION) exists"
	gh release view "$(VERSION)" --repo "$(REPO)" >/dev/null 2>&1 || \
	  gh release create "$(VERSION)" --repo "$(REPO)" --draft \
	    --title "Zoho Desktop Widget $(VERSION)" \
	    --notes "See assets below to download and install."
	echo ">> uploading $(HOST) artifacts + signatures"
	if [ "$(HOST)" = "macos" ]; then \
	  base=src-tauri/target/universal-apple-darwin/release/bundle; \
	else \
	  base=src-tauri/target/release/bundle; \
	fi; \
	files=$$(find "$$base" \
	  \( -name '*.AppImage' -o -name '*.AppImage.sig' \
	   -o -name '*.deb' -o -name '*.rpm' \
	   -o -name '*-setup.exe' -o -name '*-setup.exe.sig' \
	   -o -name '*.msi' -o -name '*.msi.sig' \
	   -o -name '*.app.tar.gz' -o -name '*.app.tar.gz.sig' \
	   -o -name '*.dmg' \) 2>/dev/null); \
	test -n "$$files" || { echo "no artifacts found under $$base"; exit 1; }; \
	echo "$$files"; \
	gh release upload "$(VERSION)" --repo "$(REPO)" --clobber $$files

# ---- finalize (run once, after all 3 hosts have uploaded) ----
.PHONY: latest-json
latest-json: ## Assemble + upload updater manifest from release assets
	@tmp=$$(mktemp -d); \
	echo ">> downloading .sig assets from $(VERSION)"; \
	gh release download "$(VERSION)" --repo "$(REPO)" --dir "$$tmp" --pattern '*.sig'; \
	REPO="$(REPO)" VERSION="$(VERSION)" SIGDIR="$$tmp" \
	  node -e '
	    const fs=require("fs"),path=require("path");
	    const {REPO,VERSION,SIGDIR}=process.env;
	    const base=`https://github.com/$${REPO}/releases/download/$${VERSION}`;
	    const sigs=fs.readdirSync(SIGDIR).filter(f=>f.endsWith(".sig"));
	    const read=f=>fs.readFileSync(path.join(SIGDIR,f),"utf8").trim();
	    const find=re=>sigs.find(f=>re.test(f));
	    const ent=(re)=>{const s=find(re);if(!s)return null;
	      const url=`$${base}/$${encodeURIComponent(s.replace(/\.sig$$/,""))}`;
	      return {signature:read(s),url};};
	    const plats={};
	    const lin=ent(/\.AppImage\.sig$$/);   if(lin) plats["linux-x86_64"]=lin;
	    const win=ent(/-setup\.exe\.sig$$/);   if(win) plats["windows-x86_64"]=win;
	    const mac=ent(/\.app\.tar\.gz\.sig$$/);if(mac){plats["darwin-aarch64"]=mac;plats["darwin-x86_64"]=mac;}
	    const out={version:VERSION.replace(/^v/,""),pub_date:new Date().toISOString(),
	      notes:"See the release page for details.",platforms:plats};
	    if(!Object.keys(plats).length){console.error("no platform sigs found");process.exit(1);}
	    fs.writeFileSync(path.join(SIGDIR,"latest.json"),JSON.stringify(out,null,2));
	    console.error("platforms: "+Object.keys(plats).join(", "));
	  '; \
	gh release upload "$(VERSION)" --repo "$(REPO)" --clobber "$$tmp/latest.json"; \
	echo ">> latest.json uploaded"

.PHONY: publish
publish: ## Un-draft the release so it becomes /releases/latest
	gh release edit "$(VERSION)" --repo "$(REPO)" --draft=false --latest

.PHONY: clean
clean: ## Remove build output
	rm -rf src-tauri/target/release/bundle \
	       src-tauri/target/universal-apple-darwin/release/bundle dist
