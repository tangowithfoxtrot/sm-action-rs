#!/bin/sh

################################################################
# a shim to select the correct binary to execute the sm-action #
################################################################

arch() {
  output="$(node -p process.arch)"
  if [ "$output" = "x64" ]; then
    ARCH="x86_64"
  elif [ "$output" = "arm64" ]; then
    ARCH="aarch64"
  else
    echo "Unsupported architecture: $output" >2
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
    echo "Unsupported platform: $output" >2
    exit 1
  fi

  echo "$PLATFORM"
}

# Main execution
main() {
  echo "Setting up bitwarden/sm-action"

  target_triple="$(arch)-$(os)"
  "./dist/$target_triple/sm-action"
}

# Run the script
main
