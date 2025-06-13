#!/bin/sh

# Helper functions
log_info() {
  echo
}

log_success() {
  echo
}

log_error() {
  echo "::error::$1"
  exit 1
}

log_debug() {
  echo "::debug::$1" >/dev/null
}

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
  echo "\n${colors[green]}Executing sm-action for ____${colors[reset]} 🎉"

  target_triple="$(arch)-$(os)"
  echo "./dist/$target_triple"
}

# Run the script
main
