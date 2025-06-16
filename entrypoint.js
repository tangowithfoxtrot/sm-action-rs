#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

function getArchitecture() {
  const arch = process.arch;
  if (arch === 'x64') {
    return 'x86_64';
  } else if (arch === 'arm64') {
    return 'aarch64';
  } else {
    console.error(`Unsupported architecture: ${arch}`);
    process.exit(1);
  }
}

function getPlatform() {
  const platform = process.platform;
  if (platform === 'linux') {
    return 'unknown-linux-musl';
  } else if (platform === 'darwin') {
    return 'apple-darwin';
  } else if (platform === 'win32') {
    return 'pc-windows-msvc';
  } else {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }
}

function main() {
  console.log('Setting up bitwarden/sm-action');

  const arch = getArchitecture();
  const platform = getPlatform();
  const targetTriple = `${arch}-${platform}`;

  // Determine binary name based on platform
  const binaryName = process.platform === 'win32' ? 'sm-action.exe' : 'sm-action';

  // Construct path to binary
  const actionPath = process.env.GITHUB_ACTION_PATH || __dirname;
  const binaryPath = path.join(actionPath, 'dist', targetTriple, binaryName);

  console.log(`Looking for binary at: ${binaryPath}`);

  // Check if binary exists
  if (!fs.existsSync(binaryPath)) {
    console.error(`Binary not found at: ${binaryPath}`);
    console.error(`Available files in dist:`);
    try {
      const distPath = path.join(actionPath, 'dist');
      if (fs.existsSync(distPath)) {
        const contents = fs.readdirSync(distPath);
        console.error(contents);
      } else {
        console.error('dist directory does not exist');
      }
    } catch (e) {
      console.error('Error listing dist contents:', e.message);
    }
    process.exit(1);
  }

  // Spawn the binary with all original arguments
  const child = spawn(binaryPath, process.argv.slice(2), {
    stdio: 'inherit',
    env: process.env
  });

  child.on('close', (code) => {
    process.exit(code);
  });

  child.on('error', (err) => {
    console.error(`Failed to start binary: ${err.message}`);
    process.exit(1);
  });
}

if (require.main === module) {
  main();
}

module.exports = { getArchitecture, getPlatform };
