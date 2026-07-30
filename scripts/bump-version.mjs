import { readFileSync, writeFileSync } from "fs"
import { join, dirname } from "path"
import { fileURLToPath } from "url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, "..")

const type = process.argv[2]
if (type !== "patch" && type !== "minor") {
  console.error("Uso: node scripts/bump-version.mjs <patch|minor>")
  process.exit(1)
}

const pkgPath = join(root, "package.json")
const pkg = JSON.parse(readFileSync(pkgPath, "utf-8"))
const [major, minor, patch] = pkg.version.split(".").map(Number)

let newVersion
if (type === "minor") {
  newVersion = `${major}.${minor + 1}.0`
} else {
  newVersion = `${major}.${minor}.${patch + 1}`
}

pkg.version = newVersion
writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n")

const cargoPath = join(root, "src-tauri", "Cargo.toml")
let cargo = readFileSync(cargoPath, "utf-8")
cargo = cargo.replace(/^version = "(\d+\.\d+\.\d+)"/m, `version = "${newVersion}"`)
writeFileSync(cargoPath, cargo)

function updateJsonFile(relPath) {
  const path = join(root, relPath)
  const obj = JSON.parse(readFileSync(path, "utf-8"))
  if (obj.version !== undefined) {
    obj.version = newVersion
    writeFileSync(path, JSON.stringify(obj, null, 2) + "\n")
  }
}

updateJsonFile("src-tauri/tauri.conf.json")
updateJsonFile("src-tauri/gen/android/app/src/main/assets/tauri.conf.json")

const propsPath = join(root, "src-tauri", "gen", "android", "app", "tauri.properties")
let props = readFileSync(propsPath, "utf-8")
props = props.replace(/^tauri\.android\.versionName=.*/m, `tauri.android.versionName=${newVersion}`)
props = props.replace(/^tauri\.android\.versionCode=(\d+)/m, (_, code) => `tauri.android.versionCode=${parseInt(code) + 1}`)
writeFileSync(propsPath, props)

console.log(`Version actualizada a ${newVersion}`)
