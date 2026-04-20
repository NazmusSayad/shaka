#!/usr/bin/env sh

set -eu

log() {
	printf '%s\n' "[shaka-install] $1"
}

fail() {
	printf '%s\n' "[shaka-install] $1" >&2
	exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required but not installed"
command -v tar >/dev/null 2>&1 || fail "tar is required but not installed"

if command -v sha256sum >/dev/null 2>&1; then
	checksum_file() {
		sha256sum "$1" | awk '{print $1}'
	}
elif command -v shasum >/dev/null 2>&1; then
	checksum_file() {
		shasum -a 256 "$1" | awk '{print $1}'
	}
else
	fail "sha256sum or shasum is required but not installed"
fi

log "Detecting platform and architecture"
os_name=$(uname -s)
arch_name=$(uname -m)

case "$os_name" in
Linux) os="linux" ;;
Darwin) os="macos" ;;
*) fail "Unsupported operating system: $os_name" ;;
esac

case "$arch_name" in
x86_64 | amd64) arch="x86_64" ;;
arm64 | aarch64) arch="aarch64" ;;
*) fail "Unsupported architecture: $arch_name" ;;
esac

if [ "$os" = "linux" ]; then
	target="${arch}-unknown-linux-gnu"
else
	target="${arch}-apple-darwin"
fi

log "Platform: $os_name"
log "Target: $target"

repo="NazmusSayad/shaka"
api_url="https://api.github.com/repos/${repo}/releases/latest"
install_dir="${HOME}/.local/bin"
binary_path="${install_dir}/shaka"

tmp_root=$(mktemp -d 2>/dev/null || mktemp -d -t shaka-install)
release_json="${tmp_root}/release.json"
archive_name=""
archive_url=""
checksum_url=""
checksum_name="sha256sums.txt"

cleanup() {
	rm -rf "$tmp_root"
}
trap cleanup EXIT INT TERM

log "Fetching latest release metadata"
curl -fsSL -H "User-Agent: shaka-installer" "$api_url" -o "$release_json" || fail "Failed to fetch latest release"

extract_browser_download_urls() {
	awk '{
		s = $0
		while (match(s, /"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*"/)) {
			m = substr(s, RSTART, RLENGTH)
			sub(/^.*"browser_download_url"[[:space:]]*:[[:space:]]*"/, "", m)
			sub(/"$/, "", m)
			print m
			s = substr(s, RSTART + RLENGTH)
		}
	}' "$release_json"
}

tag=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$release_json" | head -n 1)
[ -n "$tag" ] || fail "Release metadata parse failed: missing tag_name"

archive_name="shaka-${tag}-${target}.tar.gz"

archive_url=$(extract_browser_download_urls | grep -F "/${archive_name}" | head -n 1)
[ -n "$archive_url" ] || fail "Release metadata parse failed: missing asset:${archive_name}"

checksum_url=$(extract_browser_download_urls | grep -F "/${checksum_name}" | head -n 1)
[ -n "$checksum_url" ] || fail "Release metadata parse failed: missing checksum"

version=${tag#v}

log "Latest release tag: $tag"
log "Selecting asset: $archive_name"

if [ -x "$binary_path" ]; then
	log "Existing installation found at $binary_path"
	current_version=$($binary_path --version 2>/dev/null | awk '{for (i=1;i<=NF;i++) if ($i ~ /^[0-9]+\.[0-9]+\.[0-9]+$/) {print $i; exit}}' || true)
	if [ -n "$current_version" ] && [ "$current_version" = "$version" ]; then
		log "Installed version ($current_version) is already up to date"
		exit 0
	fi
	if [ -n "$current_version" ]; then
		log "Replacing installed version $current_version with $version"
	else
		log "Replacing existing shaka binary with version $version"
	fi
fi

archive_path="${tmp_root}/${archive_name}"
checksum_path="${tmp_root}/${checksum_name}"
extract_dir="${tmp_root}/extract"
mkdir -p "$extract_dir"

log "Downloading release archive"
curl -fsSL -H "User-Agent: shaka-installer" "$archive_url" -o "$archive_path" || fail "Failed to download archive"

log "Downloading checksum file"
curl -fsSL -H "User-Agent: shaka-installer" "$checksum_url" -o "$checksum_path" || fail "Failed to download checksum file"

log "Verifying archive checksum"
expected_hash=$(awk -v f="$archive_name" '$2 == f || $2 == "*" f {print $1}' "$checksum_path" | head -n 1)
[ -n "$expected_hash" ] || fail "Could not find checksum entry for $archive_name"
actual_hash=$(checksum_file "$archive_path")
[ "$expected_hash" = "$actual_hash" ] || fail "Checksum mismatch for $archive_name (expected $expected_hash, got $actual_hash)"

log "Extracting archive"
tar -xzf "$archive_path" -C "$extract_dir" || fail "Failed to extract archive"

new_binary=$(find "$extract_dir" -type f -name shaka | head -n 1)
[ -n "$new_binary" ] || fail "Extracted archive does not contain shaka binary"

log "Installing binary to $binary_path"
mkdir -p "$install_dir"
mv -f "$new_binary" "$binary_path"
chmod +x "$binary_path"

if printf '%s' ":${PATH}:" | grep -Fq ":${install_dir}:"; then
	log "Install directory already exists in PATH"
else
	log "Install directory is not in PATH"
	log "Add this to your shell profile and restart the shell:"
	log "export PATH=\"$install_dir:\$PATH\""
fi

log "Installation complete"
log "Installed: $binary_path"
log "Verify with: shaka --version"
