// Test-only LDB-v1 artifact fixture. No installed dependencies, network, child
// process or native execution. All header/library/parser bytes other than the
// frozen binding.gyp are explicitly synthetic: this proves structural checking
// only and must never be used as native-load or package-acceptance evidence.
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

export const STOCK_TREE_SITTER_BINDING = "{\n  \"targets\": [\n    {\n      \"target_name\": \"tree_sitter_runtime_binding\",\n      \"dependencies\": [\n          \"tree_sitter\",\n          \"<!(node -p \\\"require('node-addon-api').targets\\\"):node_addon_api_except\",\n      ],\n      \"sources\": [\n        \"src/binding.cc\",\n        \"src/conversions.cc\",\n        \"src/language.cc\",\n        \"src/logger.cc\",\n        \"src/lookaheaditerator.cc\",\n        \"src/node.cc\",\n        \"src/parser.cc\",\n        \"src/query.cc\",\n        \"src/tree.cc\",\n        \"src/tree_cursor.cc\",\n      ],\n      \"include_dirs\": [\n        \"vendor/tree-sitter/lib/include\",\n      ],\n      \"defines\": [\n        \"NAPI_VERSION=<(napi_build_version)\",\n      ],\n      \"cflags\": [\n        \"-std=c++17\"\n      ],\n      \"cflags_cc\": [\n        \"-std=c++17\"\n      ],\n      \"conditions\": [\n        [\"OS=='mac'\", {\n          \"xcode_settings\": {\n            \"GCC_SYMBOLS_PRIVATE_EXTERN\": \"YES\", # -fvisibility=hidden\n            \"CLANG_CXX_LANGUAGE_STANDARD\": \"c++17\",\n            \"MACOSX_DEPLOYMENT_TARGET\": \"10.9\",\n          },\n        }],\n        [\"OS=='win'\", {\n          \"msvs_settings\": {\n            \"VCCLCompilerTool\": {\n              \"AdditionalOptions\": [\n                \"/std:c++17\",\n              ],\n              \"RuntimeLibrary\": 0,\n            },\n          },\n        }],\n        [\"OS == 'linux'\", {\n          \"cflags_cc\": [\n            \"-Wno-cast-function-type\"\n          ]\n        }],\n      ]\n    },\n    {\n      \"target_name\": \"tree_sitter\",\n      \"type\": \"static_library\",\n      \"sources\": [\n        \"vendor/tree-sitter/lib/src/lib.c\"\n      ],\n      \"include_dirs\": [\n        \"vendor/tree-sitter/lib/src\",\n        \"vendor/tree-sitter/lib/include\",\n      ],\n      \"cflags\": [\n        \"-std=c11\"\n      ]\n    }\n  ],\n  \"variables\": {\n    \"runtime%\": \"node\",\n    \"openssl_fips\": \"\",\n    \"v8_enable_pointer_compression%\": 0,\n    \"v8_enable_31bit_smis_on_64bit_arch%\": 0,\n  }\n}\n";
const PATCHED_TREE_SITTER_BINDING = "{\n  \"targets\": [\n    {\n      \"target_name\": \"tree_sitter_runtime_binding\",\n      \"dependencies\": [\n          \"tree_sitter\",\n          \"<!(node -p \\\"require('node-addon-api').targets\\\"):node_addon_api_except\",\n      ],\n      \"sources\": [\n        \"src/binding.cc\",\n        \"src/conversions.cc\",\n        \"src/language.cc\",\n        \"src/logger.cc\",\n        \"src/lookaheaditerator.cc\",\n        \"src/node.cc\",\n        \"src/parser.cc\",\n        \"src/query.cc\",\n        \"src/tree.cc\",\n        \"src/tree_cursor.cc\",\n      ],\n      \"include_dirs\": [\n        \"vendor/tree-sitter/lib/include\",\n      ],\n      \"defines\": [\n        \"NAPI_VERSION=<(napi_build_version)\",\n      ],\n      \"conditions\": [\n        [\"OS=='mac'\", {\n          \"xcode_settings\": {\n            \"GCC_SYMBOLS_PRIVATE_EXTERN\": \"YES\", # -fvisibility=hidden\n            \"CLANG_CXX_LANGUAGE_STANDARD\": \"<(cxxstd)\",\n            \"MACOSX_DEPLOYMENT_TARGET\": \"10.9\",\n          },\n        }],\n        [\"OS=='win'\", {\n          \"msvs_settings\": {\n            \"VCCLCompilerTool\": {\n              \"AdditionalOptions\": [\n                \"/std:<(cxxstd)\",\n              ],\n              \"RuntimeLibrary\": 0,\n            },\n          },\n        }],\n        [\"OS == 'linux'\", {\n          \"cflags_cc\": [\n            \"-std=<(cxxstd)\",\n            \"-Wno-cast-function-type\",\n          ]\n        }],\n      ]\n    },\n    {\n      \"target_name\": \"tree_sitter\",\n      \"type\": \"static_library\",\n      \"sources\": [\n        \"vendor/tree-sitter/lib/src/lib.c\"\n      ],\n      \"include_dirs\": [\n        \"vendor/tree-sitter/lib/src\",\n        \"vendor/tree-sitter/lib/include\",\n      ],\n      \"cflags\": [\n        \"-std=c11\"\n      ]\n    }\n  ],\n  \"variables\": {\n    \"runtime%\": \"node\",\n    \"openssl_fips\": \"\",\n    \"v8_enable_pointer_compression%\": 0,\n    \"v8_enable_31bit_smis_on_64bit_arch%\": 0,\n    \"cxxstd%\": \"<!(node -p \\\"parseInt(process.env.npm_config_target ?? process.versions.node) < 22 ? 'c++17' : 'c++20'\\\")\",\n  }\n}\n";
const PACKAGES = [
  ["tree-sitter", "0.21.1", "LICENSE"],
  ["tree-sitter-bash", "0.21.0", "LICENSE"],
  ["node-addon-api", "8.5.0", "LICENSE.md"],
  ["node-gyp-build", "4.8.4", "LICENSE"],
  ["web-tree-sitter", "0.21.0", "LICENSE"],
];
function hash(bytes) { return createHash("sha256").update(bytes).digest("hex"); }
function identity(relative, input) {
  const bytes = Buffer.isBuffer(input) ? input : Buffer.from(input);
  return { path: relative, bytes: bytes.length, sha256: hash(bytes) };
}
function sort(records) { return records.sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0); }

export function createLocalExecDependencyFixture({ desktopRoot, domain }) {
  if (domain !== "node" && domain !== "electron") throw new Error("Fixture requires an explicit runtime domain");
  const desktopPackage = JSON.parse(readFileSync(path.join(desktopRoot, "package.json"), "utf8"));
  const lockBytes = readFileSync(path.join(desktopRoot, "package-lock.json"));
  const lock = JSON.parse(lockBytes);
  const stageRoot = path.join(desktopRoot, "dist", domain === "node" ? "node-deps" : "deps");
  mkdirSync(path.dirname(stageRoot), { recursive: true });
  mkdirSync(stageRoot);
  const runtime = {
    version: domain === "electron" ? desktopPackage.devDependencies.electron : process.versions.node,
    node: process.versions.node, modules: process.versions.modules, platform: process.platform, arch: process.arch,
  };
  const files = new Map();
  const sources = [];
  for (const [name, version, licenseFile] of PACKAGES) {
    const contents = new Map([
      ["package.json", JSON.stringify({ name, version, license: "MIT" }) + "\n"],
      [licenseFile, "SYNTHETIC LICENSE PLACEHOLDER: structural fixture only.\n"],
      ["README.md", "SYNTHETIC package documentation: structural fixture only.\n"],
    ]);
    if (name === "tree-sitter") {
      contents.set("binding.gyp", PATCHED_TREE_SITTER_BINDING);
      contents.set("src/binding.cc", "// SYNTHETIC source; not compiled.\n");
      contents.set("vendor/tree-sitter/lib/src/lib.c", "/* SYNTHETIC source; not compiled. */\n");
      contents.set("vendor/tree-sitter/LICENSE", "SYNTHETIC vendor license placeholder.\n");
      contents.set("vendor/tree-sitter/lib/src/unicode/LICENSE", "SYNTHETIC Unicode license placeholder.\n");
    }
    if (name === "tree-sitter-bash") {
      contents.set("src/parser.c", "/* SYNTHETIC parser source; not compiled. */\n");
      contents.set("src/scanner.c", "/* SYNTHETIC scanner source; not compiled. */\n");
    }
    for (const [relative, bytes] of contents) {
      files.set(name + "/" + relative, Buffer.from(bytes));
      sources.push(identity("source/" + name + "/" + relative, name === "tree-sitter" && relative === "binding.gyp" ? STOCK_TREE_SITTER_BINDING : bytes));
      if (PACKAGES.findIndex(p => p[0] === name) >= 2) files.set("node_modules/" + name + "/" + relative, Buffer.from(bytes));
    }
  }
  files.set("tree-sitter/build/Release/tree_sitter_runtime_binding.node", Buffer.from("SYNTHETIC NOT A NATIVE BINARY\n"));
  files.set("tree-sitter-bash/build/Release/tree_sitter_bash_binding.node", Buffer.from("SYNTHETIC NOT A NATIVE LANGUAGE BINARY\n"));
  const archiveName = "node-v" + runtime.version + "-headers.tar.gz";
  const archive = Buffer.from("SYNTHETIC NOT A HEADER ARCHIVE: " + domain + "\n");
  files.set(".headers/" + archiveName, archive);
  let checksum = hash(archive) + "  " + archiveName + "\n";
  const [major, minor, patch] = runtime.node.split(".");
  const headerRoot = ".headers/" + runtime.version + "/";
  files.set(headerRoot + "include/node/node_version.h", Buffer.from([
    "/* SYNTHETIC header declarations: structural checking only. */",
    "#define NODE_MAJOR_VERSION " + major,
    "#define NODE_MINOR_VERSION " + minor,
    "#define NODE_PATCH_VERSION " + patch,
    "#define NODE_MODULE_VERSION " + runtime.modules,
    "",
  ].join("\n")));
  files.set(headerRoot + "include/node/config.gypi", Buffer.from('{"variables":{"synthetic_fixture":true}}\n'));
  if (runtime.platform === "win32") {
    const library = Buffer.from("SYNTHETIC NOT A WINDOWS IMPORT LIBRARY\n");
    files.set(headerRoot + runtime.arch + "/node.lib", library);
    checksum += hash(library) + "  win-" + (runtime.arch === "ia32" ? "x86" : runtime.arch) + "/node.lib\n";
  }
  files.set(".headers/SHASUMS256.txt", Buffer.from(checksum));
  for (const [relative, bytes] of files) {
    mkdirSync(path.dirname(path.join(stageRoot, relative)), { recursive: true });
    writeFileSync(path.join(stageRoot, relative), bytes);
  }
  const stageFiles = sort([...files].map(([relative, bytes]) => identity(relative, bytes)));
  const headerFiles = stageFiles.filter(f => f.path.startsWith(".headers/"));
  const inputs = [
    ...sources,
    ...headerFiles.map(f => ({ ...f, path: "headers/" + f.path.slice(".headers/".length) })),
    identity("scripts/build-local-exec-dependencies.mjs", readFileSync(path.join(desktopRoot, "scripts/build-local-exec-dependencies.mjs"))),
    identity("package.json", readFileSync(path.join(desktopRoot, "package.json"))),
    identity("package-lock.json", lockBytes),
    identity("runtime/host-node", "SYNTHETIC host runtime identity"),
    identity("runtime/" + domain + "-executable", "SYNTHETIC selected runtime identity"),
    identity("configuration/runtime.json", JSON.stringify(runtime)),
    identity("configuration/host-node-config.json", "SYNTHETIC host configuration"),
    identity("configuration/build-environment.json", "SYNTHETIC build configuration"),
    ...["package.json", "bin/node-gyp.js", "lib/install.js", "lib/download.js", "lib/process-release.js"]
      .map(p => identity("tool/node_modules/node-gyp/" + p, "SYNTHETIC installed node-gyp input: " + p)),
  ];
  sort(inputs);
  const base = (domain === "electron" ? "https://www.electronjs.org/headers" : "https://nodejs.org/dist") + "/v" + runtime.version + "/";
  const receipt = {
    contract: "LDB-v1", domain, runtime, package_lock_sha256: hash(lockBytes),
    build_input_sha256: hash(JSON.stringify(inputs)), build_inputs: inputs,
    packages: PACKAGES.map(([name, version], index) => {
      const entry = lock.packages["node_modules/" + name];
      if (entry.version !== version) throw new Error("Fixture requires the actual locked dependency: " + name);
      return {
        name, version, resolved: entry.resolved, integrity: entry.integrity, license: entry.license,
        files: stageFiles.filter(f => f.path.startsWith(name + "/") || (index >= 2 && f.path.startsWith("node_modules/" + name + "/"))),
      };
    }),
    headers: {
      url: base + archiveName, version: runtime.version, checksum_url: base + "SHASUMS256.txt",
      archive_sha256: hash(archive), checksum_sha256: hash(Buffer.from(checksum)), files: headerFiles,
    },
    native_outputs: stageFiles.filter(f => f.path.endsWith(".node")), stage_files: stageFiles,
    load_probe: { runtime, exit: 0, parser_language_loaded: true },
  };
  writeFileSync(path.join(stageRoot, "local-exec-dependencies.json"), JSON.stringify(receipt, null, 2) + "\n");
  return receipt;
}
