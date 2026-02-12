#!/usr/bin/env node
import { readFileSync, writeFileSync, existsSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = join(__dirname, '..');

const packageJson = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf-8'));
const version = packageJson.version;

console.log(`Syncing version to ${version}...`);

const tauriConfPath = join(rootDir, 'src-tauri', 'tauri.conf.json');
const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf-8'));
tauriConf.version = version;
writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n');

const cargoTomlPath = join(rootDir, 'src-tauri', 'Cargo.toml');
let cargoToml = readFileSync(cargoTomlPath, 'utf-8');
cargoToml = cargoToml.replace(/^version = ".*"$/m, `version = "${version}"`);
writeFileSync(cargoTomlPath, cargoToml);

const appSveltePath = join(rootDir, 'src', 'App.svelte');
let appSvelte = readFileSync(appSveltePath, 'utf-8');
appSvelte = appSvelte.replace(/<span>v[\d.]+<\/span>/g, `<span>v${version}</span>`);
writeFileSync(appSveltePath, appSvelte);

const pkgbuildPath = join(rootDir, 'AUR', 'PKGBUILD');
if (existsSync(pkgbuildPath)) {
  let pkgbuild = readFileSync(pkgbuildPath, 'utf-8');
  pkgbuild = pkgbuild.replace(/^pkgver=.*$/m, `pkgver=${version}`);
  writeFileSync(pkgbuildPath, pkgbuild);
}

const pkgbuildBinPath = join(rootDir, 'AUR', 'PKGBUILD-bin');
if (existsSync(pkgbuildBinPath)) {
  let pkgbuildBin = readFileSync(pkgbuildBinPath, 'utf-8');
  pkgbuildBin = pkgbuildBin.replace(/^pkgver=.*$/m, `pkgver=${version}`);
  writeFileSync(pkgbuildBinPath, pkgbuildBin);
}
