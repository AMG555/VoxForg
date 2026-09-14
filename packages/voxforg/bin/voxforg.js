#!/usr/bin/env node

/**
 * VoxForg Standalone Native Runner
 * Automatically resolves, downloads, and executes the platform-specific
 * VoxForg binary with embedded workstation UI and neural runtime.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');
const { spawn } = require('child_process');

const REPO = 'AMG555/VoxForg';
const DEFAULT_VERSION = 'v0.1.0';

function getTargetInfo() {
  const platform = process.platform;
  const arch = process.arch;

  let osName = '';
  let archName = '';
  let ext = '';

  switch (platform) {
    case 'win32':
      osName = 'windows';
      ext = '.exe';
      break;
    case 'darwin':
      osName = 'macos';
      break;
    case 'linux':
      osName = 'linux';
      break;
    default:
      console.error(`[voxforg] Unsupported platform: ${platform}`);
      process.exit(1);
  }

  switch (arch) {
    case 'x64':
      archName = 'x86_64';
      break;
    case 'arm64':
      archName = 'aarch64';
      break;
    default:
      console.error(`[voxforg] Unsupported architecture: ${arch}`);
      process.exit(1);
  }

  const binaryName = `voxforg-${osName}-${archName}${ext}`;
  return { binaryName, osName, archName, ext };
}

function downloadBinary(url, destPath) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(destPath);
    console.log(`[voxforg] Downloading native workstation runtime from ${url}...`);

    function makeRequest(currentUrl) {
      https.get(currentUrl, { headers: { 'User-Agent': 'VoxForg-Runner' } }, (response) => {
        if (response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
          makeRequest(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          file.close();
          fs.unlink(destPath, () => {});
          reject(new Error(`Server returned HTTP ${response.statusCode}`));
          return;
        }

        let downloadedBytes = 0;
        const totalBytes = parseInt(response.headers['content-length'] || '0', 10);

        response.on('data', (chunk) => {
          downloadedBytes += chunk.length;
          if (totalBytes > 0 && process.stdout.isTTY) {
            const pct = Math.round((downloadedBytes / totalBytes) * 100);
            process.stdout.write(`\r[voxforg] Downloading: ${pct}% (${(downloadedBytes / 1048576).toFixed(1)} MB)`);
          }
        });

        response.pipe(file);

        file.on('finish', () => {
          file.close();
          if (process.stdout.isTTY) process.stdout.write('\n');
          console.log('[voxforg] Download completed successfully.');
          resolve();
        });
      }).on('error', (err) => {
        file.close();
        fs.unlink(destPath, () => {});
        reject(err);
      });
    }

    makeRequest(url);
  });
}

async function main() {
  const { binaryName, ext } = getTargetInfo();
  const cacheDir = path.join(os.homedir(), '.voxforg', 'bin');
  const binaryPath = path.join(cacheDir, `voxforg${ext}`);

  if (!fs.existsSync(binaryPath)) {
    fs.mkdirSync(cacheDir, { recursive: true });
    const downloadUrl = `https://github.com/${REPO}/releases/download/${DEFAULT_VERSION}/${binaryName}`;

    try {
      await downloadBinary(downloadUrl, binaryPath);
      if (process.platform !== 'win32') {
        fs.chmodSync(binaryPath, 0o755);
      }
    } catch (err) {
      console.warn(`[voxforg] Pre-built binary download not available (${err.message}).`);
      console.log('[voxforg] Checking for local cargo installation...');

      // Fallback: Check if user has voxforg in PATH or cargo
      try {
        const check = spawn(process.platform === 'win32' ? 'where' : 'which', ['voxforg']);
        check.on('close', (code) => {
          if (code === 0) {
            runBinary('voxforg');
          } else {
            console.error('[voxforg] Please run: cargo install --git https://github.com/AMG555/VoxForg.git voxforg-cli');
            process.exit(1);
          }
        });
        return;
      } catch {
        process.exit(1);
      }
    }
  }

  runBinary(binaryPath);
}

function runBinary(executable) {
  let args = process.argv.slice(2);
  // Default to serve --open if no arguments specified
  if (args.length === 0) {
    args = ['serve', '--open'];
  }

  const child = spawn(executable, args, { stdio: 'inherit' });

  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code || 0);
    }
  });

  child.on('error', (err) => {
    console.error(`[voxforg] Process execution error: ${err.message}`);
    process.exit(1);
  });
}

main().catch((err) => {
  console.error(`[voxforg] Fatal error: ${err.message}`);
  process.exit(1);
});
