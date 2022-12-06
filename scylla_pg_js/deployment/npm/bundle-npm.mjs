import {copyFileSync, mkdirSync, readFileSync, writeFileSync, readdirSync, statSync, rmSync} from "fs"
import path from "path"
import { getCliArg, copyFiles } from "./bundle-utils.mjs"

function copyPackageFiles(bundleDir, packageFiles) {
  // delete existing bundle directory
  rmSync(bundleDir, { recursive: true, force: true });
  // create bundle dir
  mkdirSync(bundleDir, {recursive: true});
  // Copy package files to bundle dir
  packageFiles.forEach(f => copyFileSync(path.resolve(f), path.resolve(bundleDir, f)));

}

function versionPackage(packageName, nextVersion){
  const libName = path.basename(packageName)
  const packageJsonPath = path.resolve("package.json")
  const packageLockJsonPath = path.resolve("package-lock.json")
  // change version
  const paths = [packageJsonPath, packageLockJsonPath]
  paths.forEach(p => {
    const json = JSON.parse(readFileSync(p, {encoding: "utf-8"}))
    json.version = nextVersion
    json.name = json.name.replace(/\/.+$/i, "/" + libName)
    writeFileSync(p, JSON.stringify(json, undefined, 2), {encoding: "utf-8"})
  })
}

const bundleDir = "build/.bundles"
const packageFiles = [
    ".npmrc",
    "package.json",
    "index.d.ts",
    "scylla.node",
    "index.cjs",
    "package-lock.json"
];
copyPackageFiles(bundleDir, packageFiles);
versionPackage("pg_js", getCliArg("ver"))

