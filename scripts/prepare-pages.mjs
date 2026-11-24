#!/usr/bin/env node
import { copyFileSync, existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const publicDir = process.argv[2];
const rawBase = process.argv[3] ?? "";
const basePrefix = rawBase === "/" ? "" : rawBase.replace(/\/+$/, "");

if (!publicDir || !existsSync(join(publicDir, "index.html"))) {
  console.error("usage: node scripts/prepare-pages.mjs <public_dir> [base_path]");
  process.exit(1);
}

const assetsDir = join(publicDir, "assets");
const assetFiles = existsSync(assetsDir) ? readdirSync(assetsDir).sort() : [];
const cssFile = assetFiles.find((file) => /^main.*\.css$/.test(file));
const wasmFile = assetFiles.find((file) => /\.wasm$/.test(file));
const iconFile = assetFiles.find((file) => /^favicon.*\.svg$/.test(file));

const headLinks = [
  iconFile ? `<link rel="icon" type="image/svg+xml" href="${basePrefix}/assets/${iconFile}">` : "",
  cssFile ? `<link rel="stylesheet" href="${basePrefix}/assets/${cssFile}">` : "",
  wasmFile
    ? `<link rel="preload" href="${basePrefix}/assets/${wasmFile}" as="fetch" type="application/wasm" crossorigin>`
    : "",
]
  .filter(Boolean)
  .join("\n    ");

const bootShell = `<style>#trace-boot{min-height:100dvh;display:grid;place-items:center;font:14px/1.5 -apple-system,BlinkMacSystemFont,"PingFang SC",sans-serif;background:#f5f7f9;color:#334155}:has([data-trace-root]) #trace-boot{display:none}</style><div id="trace-boot" aria-hidden="true">Loading Dioxus Trace Viewer (WASM)...</div>`;

const indexPath = join(publicDir, "index.html");
let html = readFileSync(indexPath, "utf8");
if (headLinks && !html.includes(`href="${basePrefix}/assets/${cssFile}"`)) {
  html = html.replace("</head>", `    ${headLinks}\n  </head>`);
}
if (!html.includes('id="trace-boot"')) {
  html = html.replace('<div id="main"></div>', `${bootShell}\n    <div id="main"></div>`);
}
writeFileSync(indexPath, html);
copyFileSync(indexPath, join(publicDir, "404.html"));
writeFileSync(join(publicDir, ".nojekyll"), "");

console.log(
  `Prepared Pages output in ${publicDir} (base=${basePrefix || "/"}, css=${cssFile ?? "none"}, wasm=${wasmFile ?? "none"})`,
);
