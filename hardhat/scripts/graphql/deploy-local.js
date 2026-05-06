#!/usr/bin/env node
// deploy-local.js
// Uploads a built subgraph to IPFS and deploys via graph-node admin RPC.
// Replaces `graph deploy` to avoid its interactive TTY prompt.
// Requires: Node 18+ (native fetch). No extra deps.

'use strict';

const fs   = require('fs');
const path = require('path');

const IPFS_URL      = process.env.IPFS_URL       || 'http://ipfs:5001';
const GRAPH_NODE    = process.env.GRAPH_NODE_URL  || 'http://graph-node:8020';
const SUBGRAPH_NAME = process.env.SUBGRAPH_NAME   || 'loan-machine';
const VERSION_LABEL = process.env.VERSION_LABEL   || 'v0.0.1';
const BUILD_DIR     = path.resolve(process.env.BUILD_DIR || './build');

// ── IPFS helpers ──────────────────────────────────────────────────────────────

async function ipfsAdd(content, filename) {
  const boundary = 'boundary' + Math.random().toString(16).slice(2);
  const body = Buffer.concat([
    Buffer.from(
      `--${boundary}\r\n` +
      `Content-Disposition: form-data; name="file"; filename="${filename}"\r\n` +
      `Content-Type: application/octet-stream\r\n\r\n`
    ),
    Buffer.isBuffer(content) ? content : Buffer.from(content, 'utf8'),
    Buffer.from(`\r\n--${boundary}--\r\n`),
  ]);

  const res = await fetch(`${IPFS_URL}/api/v0/add?pin=true`, {
    method:  'POST',
    headers: { 'Content-Type': `multipart/form-data; boundary=${boundary}` },
    body,
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`IPFS add failed for ${filename}: ${res.status} ${text}`);
  }

  const json = await res.json();
  return json.Hash;
}

// ── Manifest processing ───────────────────────────────────────────────────────

// Returns every `file: <path>` value found in the manifest yaml text.
function extractFilePaths(manifest) {
  const matches = [...manifest.matchAll(/^\s+file:\s+(\S+)/gm)];
  return [...new Set(matches.map(m => m[1].trim()))];
}

async function uploadManifestFiles(manifest) {
  const filePaths = extractFilePaths(manifest);
  let updated = manifest;

  for (const relPath of filePaths) {
    const absPath = path.join(BUILD_DIR, relPath);

    if (!fs.existsSync(absPath)) {
      console.warn(`  ⚠ skipping missing file: ${relPath}`);
      continue;
    }

    const content  = fs.readFileSync(absPath);
    const filename = path.basename(relPath);
    const hash     = await ipfsAdd(content, filename);
    const ipfsPath = `/ipfs/${hash}`;

    console.log(`  ✓ ${relPath} → ${ipfsPath}`);

    // Replace all occurrences of this path in the manifest
    updated = updated.split(relPath).join(ipfsPath);
  }

  return updated;
}

// ── graph-node admin RPC ──────────────────────────────────────────────────────

async function graphNodeDeploy(ipfsHash) {
  const res = await fetch(`${GRAPH_NODE}/`, {
    method:  'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      method:  'subgraph_deploy',
      params:  {
        name:          SUBGRAPH_NAME,
        ipfs_hash:     ipfsHash,
        version_label: VERSION_LABEL,
      },
      id: '1',
    }),
  });

  if (!res.ok) {
    throw new Error(`graph-node RPC HTTP error: ${res.status}`);
  }

  const json = await res.json();

  if (json.error) {
    throw new Error(`graph-node RPC error: ${JSON.stringify(json.error)}`);
  }

  return json.result;
}

// ── Main ──────────────────────────────────────────────────────────────────────

async function main() {
  const manifestPath = path.join(BUILD_DIR, 'subgraph.yaml');

  if (!fs.existsSync(manifestPath)) {
    console.error(`✗ build/subgraph.yaml not found — run 'graph build' first`);
    process.exit(1);
  }

  console.log('=== Uploading files to IPFS ===');
  const rawManifest     = fs.readFileSync(manifestPath, 'utf8');
  const updatedManifest = await uploadManifestFiles(rawManifest);

  console.log('=== Uploading manifest ===');
  const manifestHash = await ipfsAdd(Buffer.from(updatedManifest, 'utf8'), 'subgraph.yaml');
  console.log(`  ✓ manifest → /ipfs/${manifestHash}`);

  console.log('=== Deploying to graph-node ===');
  const result = await graphNodeDeploy(manifestHash);
  console.log('  ✓ deployed:', JSON.stringify(result, null, 2));

  console.log('');
  console.log('======================================');
  console.log('  SUBGRAPH DEPLOYED SUCCESSFULLY');
  console.log('======================================');
  console.log(`  GraphQL : http://localhost:8000/subgraphs/name/${SUBGRAPH_NAME}`);
  console.log('  Status  : http://localhost:8030/graphql');
  console.log('======================================');
}

main().catch(err => {
  console.error('✗', err.message);
  process.exit(1);
});