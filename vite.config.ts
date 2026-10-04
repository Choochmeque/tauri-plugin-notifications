import { readFileSync } from "node:fs";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const rootDir = dirname(fileURLToPath(import.meta.url));

type StringRecord = Readonly<Record<string, string>>;

type PackageManifest = {
  readonly exports: { readonly import: string; readonly require: string };
  readonly dependencies?: StringRecord;
  readonly peerDependencies?: StringRecord;
};

function isStringRecord(value: unknown): value is StringRecord {
  return (
    typeof value === "object" &&
    value !== null &&
    Object.values(value).every((entry) => typeof entry === "string")
  );
}

function isPackageManifest(value: unknown): value is PackageManifest {
  if (typeof value !== "object" || value === null || !("exports" in value)) {
    return false;
  }
  const { exports } = value;
  if (typeof exports !== "object" || exports === null) {
    return false;
  }
  if (!("import" in exports) || typeof exports.import !== "string") {
    return false;
  }
  if (!("require" in exports) || typeof exports.require !== "string") {
    return false;
  }
  if ("dependencies" in value && !isStringRecord(value.dependencies)) {
    return false;
  }
  return (
    !("peerDependencies" in value) || isStringRecord(value.peerDependencies)
  );
}

const manifest: unknown = JSON.parse(
  readFileSync(join(rootDir, "package.json"), "utf8"),
);
if (!isPackageManifest(manifest)) {
  throw new Error(
    "package.json must declare string exports.import / exports.require entry points",
  );
}

// The published entry points decide where the bundles go, so that package.json
// stays the single source of truth for the output paths.
const esmFile = basename(manifest.exports.import);
const cjsFile = basename(manifest.exports.require);

export default defineConfig({
  build: {
    target: "es2021",
    outDir: dirname(manifest.exports.import),
    emptyOutDir: true,
    minify: false,
    sourcemap: false,
    lib: {
      entry: join(rootDir, "guest-js", "index.ts"),
      formats: ["es", "cjs"],
      fileName: (format): string => (format === "cjs" ? cjsFile : esmFile),
    },
    rolldownOptions: {
      external: [
        /^@tauri-apps\/api/,
        ...Object.keys(manifest.dependencies ?? {}),
        ...Object.keys(manifest.peerDependencies ?? {}),
      ],
    },
  },
});
