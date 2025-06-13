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
    error "Unsupported architecture: $output"
  fi

  echo "$ARCH"
}

os() {
  output="$(node -p process.platform)"
  if [ "$output" = "linux" ]; then
    PLATFORM="unknown-linux-musl"
  elif [ "$output" = "darwin" ]; then
    PLATFORM="apple-darwin"
  else
    error "Unsupported platform: $output"
  fi

  echo "$PLATFORM"
}

# Main execution
main() {
  log_info "Setting up bitwarden/sm-action"
  echo "Executing sm-action for ____"

  target_triple="$(arch)-$(os)"
  "./dist/$target_triple/sm-action"
}

# Run the script
main
