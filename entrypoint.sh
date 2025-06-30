#!/bin/sh

################################################################
# Download and execute pre-built sm-action binary from GitHub releases
################################################################

arch() {
  output="$(node -p process.arch)"
  if [ "$output" = "x64" ]; then
    ARCH="x86_64"
  elif [ "$output" = "arm64" ]; then
    ARCH="aarch64"
  else
    echo "Unsupported architecture: $output" >&2
    exit 1
  fi

  echo "$ARCH"
}

os() {
  output="$(node -p process.platform)"
  if [ "$output" = "linux" ]; then
    PLATFORM="unknown-linux-musl"
  elif [ "$output" = "darwin" ]; then
    PLATFORM="apple-darwin"
  elif [ "$output" = "win32" ]; then
    PLATFORM="pc-windows-msvc"
  else
    echo "Unsupported platform: $output" >&2
    exit 1
  fi

  echo "$PLATFORM"
}

download_binary() {
  target_triple="$1"

  # Determine file extension and binary name
  if echo "$target_triple" | grep -q "windows"; then
    file_ext="zip"
    binary_name="sm-action.exe"
  else
    file_ext="tar.gz"
    binary_name="sm-action"
  fi

  archive_name="sm-action-${target_triple}.${file_ext}"
  download_url="https://github.com/${GITHUB_REPOSITORY}/releases/latest/download/${archive_name}"

  echo "Downloading ${archive_name} from latest release..."

  # Download the archive
  if ! curl -fsSL "$download_url" -o "$archive_name"; then
    echo "Failed to download from latest release, trying to build locally..."
    return 1
  fi

  # Extract the binary
  if [ "$file_ext" = "zip" ]; then
    unzip -q "$archive_name" "$binary_name"
  else
    tar -xzf "$archive_name" "$binary_name"
  fi

  chmod +x "$binary_name"
  echo "Successfully downloaded and extracted $binary_name"
  return 0
}

build_fallback() {
  target_triple="$1"
  echo "Building sm-action locally as fallback..."

  # Check if we have Rust installed
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: No pre-built binary available and Rust not installed"
    exit 1
  fi

  # Ensure we have the correct target installed
  rustup target add "$target_triple"

  # Build for current platform
  cargo build --release --target "$target_triple"

  if echo "$target_triple" | grep -q "windows"; then
    cp "target/${target_triple}/release/sm-action.exe" .
  else
    cp "target/${target_triple}/release/sm-action" .
    chmod +x sm-action
  fi
}

# Main execution
main() {
  echo "Setting up bitwarden/sm-action"

  target_triple="$(arch)-$(os)"

  # Try to download pre-built binary first
  if ! download_binary "$target_triple"; then
    build_fallback "$target_triple"
  fi

  # Execute the binary
  if echo "$target_triple" | grep -q "windows"; then
    ./sm-action.exe
  else
    ./sm-action
  fi
}

# Run the script
main
