import fs from "fs";
import path from "path";

const versionFilePath = path.join(process.cwd(), '..', 'VERSION');
const packageJsonPath = path.join(process.cwd(), 'package.json');

const versionFile = fs.readFileSync(versionFilePath, 'utf-8').trim();
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf-8'));
const packageVersion = packageJson.version;

if (versionFile !== packageVersion) {
  console.error(`Version mismatch: VERSION=${versionFile}, package.json=${packageVersion}`);
  process.exit(1);
}

console.info(`Package version: ${packageVersion}`);
console.info(`Version check passed: ${versionFile}`);
