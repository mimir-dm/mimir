#!/usr/bin/env node
/**
 * Build the mimir-mcp sidecar binary for Tauri bundling.
 * Cross-platform script that works on macOS, Linux, and Windows.
 */

import { execSync } from 'child_process';
import { existsSync, mkdirSync, copyFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { platform, arch } from 'os';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const projectRoot = join(__dirname, '..');
const binariesDir = join(projectRoot, 'crates', 'mimir-dm', 'binaries');

/**
 * Detect the target triple for the current platform or from environment.
 */
function detectTarget() {
  // Priority 1: Tauri's target triple (set during tauri build)
  if (process.env.TAURI_ENV_TARGET_TRIPLE) {
    return process.env.TAURI_ENV_TARGET_TRIPLE;
  }

  // Priority 2: Command line argument
  if (process.argv[2]) {
    return process.argv[2];
  }

  // Priority 3: Auto-detect from platform
  const os = platform();
  const cpuArch = arch();

  const targetMap = {
    darwin: {
      arm64: 'aarch64-apple-darwin',
      x64: 'x86_64-apple-darwin',
    },
    linux: {
      arm64: 'aarch64-unknown-linux-gnu',
      x64: 'x86_64-unknown-linux-gnu',
    },
    win32: {
      arm64: 'aarch64-pc-windows-msvc',
      x64: 'x86_64-pc-windows-msvc',
    },
  };

  const osTargets = targetMap[os];
  if (!osTargets) {
    console.error(`Unsupported OS: ${os}`);
    process.exit(1);
  }

  const target = osTargets[cpuArch];
  if (!target) {
    console.error(`Unsupported architecture: ${cpuArch} on ${os}`);
    process.exit(1);
  }

  return target;
}

function main() {
  const target = detectTarget();
  const isWindows = platform() === 'win32';
  const binaryName = isWindows ? 'mimir-mcp.exe' : 'mimir-mcp';
  const targetBinaryName = isWindows ? `mimir-mcp-${target}.exe` : `mimir-mcp-${target}`;

  console.log(`Building mimir-mcp for target: ${target}`);

  // Build mimir-mcp in release mode
  try {
    execSync(`cargo build --release -p mimir-dm-mcp --target ${target}`, {
      cwd: projectRoot,
      stdio: 'inherit',
    });
  } catch (error) {
    console.error('Cargo build failed');
    process.exit(1);
  }

  // Ensure binaries directory exists
  if (!existsSync(binariesDir)) {
    mkdirSync(binariesDir, { recursive: true });
  }

  // Copy binary with target suffix
  const source = join(projectRoot, 'target', target, 'release', binaryName);
  const dest = join(binariesDir, targetBinaryName);

  if (existsSync(source)) {
    copyFileSync(source, dest);
    console.log(`Copied: ${dest}`);
  } else {
    console.error(`Error: Binary not found at ${source}`);
    process.exit(1);
  }

  console.log('Sidecar build complete!');
}

main();
