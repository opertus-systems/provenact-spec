import fs from "node:fs";
import path from "node:path";
import Ajv from "ajv";
import addFormats from "ajv-formats";
import YAML from "yaml";

const ROOT = process.cwd();
const ajv = new Ajv({ allErrors: true, strict: false });
addFormats(ajv);

function loadText(file) {
  return fs.readFileSync(path.join(ROOT, file), "utf8");
}

function loadDoc(file) {
  const text = loadText(file);
  if (file.endsWith(".yaml") || file.endsWith(".yml")) {
    return YAML.parse(text);
  }
  return JSON.parse(text);
}

function listFiles(dir, exts = [".json", ".yaml", ".yml"]) {
  const abs = path.join(ROOT, dir);
  if (!fs.existsSync(abs)) return [];
  const files = fs.readdirSync(abs)
    .filter((name) => exts.some((ext) => name.endsWith(ext)))
    .map((name) => path.join(dir, name))
    .sort();
  return files;
}

function assertSchemaGroup({ schemaFile, goodDir, badDir, semanticBad = [] }) {
  const schema = loadDoc(schemaFile);
  const validate = ajv.compile(schema);

  let checks = 0;
  const goodFiles = listFiles(goodDir);
  const badFiles = listFiles(badDir);
  const semanticBadSet = new Set(semanticBad);

  for (const file of goodFiles) {
    checks += 1;
    const ok = validate(loadDoc(file));
    if (!ok) {
      throw new Error(`expected valid: ${file}: ${ajv.errorsText(validate.errors)}`);
    }
  }

  for (const file of badFiles) {
    if (semanticBadSet.has(file)) {
      console.log(`SKIP semantic-only negative ${file}`);
      continue;
    }
    checks += 1;
    const ok = validate(loadDoc(file));
    if (ok) {
      throw new Error(`expected invalid: ${file}`);
    }
  }

  console.log(`OK schema ${schemaFile} (good=${goodFiles.length} bad=${badFiles.length})`);
  return checks;
}

function normalizeFsPath(value) {
  if (!value.startsWith("/")) return null;
  if (value.includes("\0")) return null;
  const parts = [];
  for (const segment of value.split("/")) {
    if (!segment || segment === ".") continue;
    if (segment === "..") return null;
    parts.push(segment);
  }
  return `/${parts.join("/")}`;
}

function isWithinPrefix(candidate, prefix) {
  if (prefix === "/") return candidate.startsWith("/");
  if (candidate === prefix) return true;
  return candidate.startsWith(`${prefix}/`);
}

function normalizeUriPath(value) {
  const raw = value === "" ? "/" : value;
  if (raw.includes("\\") || /%[0-9A-Fa-f]{2}/.test(raw)) return null;
  return normalizeFsPath(raw);
}

function netUriWithinPrefix(requested, allowed) {
  if (requested.protocol !== allowed.protocol) return false;
  if (requested.hostname !== allowed.hostname) return false;
  if (requested.port !== allowed.port) return false;
  if (requested.username !== allowed.username || requested.password !== allowed.password) return false;
  if (requested.hash || allowed.search || allowed.hash) return false;

  const requestedPath = normalizeUriPath(requested.pathname);
  const allowedPath = normalizeUriPath(allowed.pathname);
  if (!requestedPath || !allowedPath) return false;
  return isWithinPrefix(requestedPath, allowedPath);
}

function isValidEnvName(value) {
  return /^[A-Z_][A-Z0-9_]*$/.test(value);
}

function evaluateCapability(policy, capability) {
  const ceiling = policy.capability_ceiling ?? {};
  const kind = capability.kind;
  const value = capability.value;

  if (kind === "exec") {
    return value === "true" && ceiling.exec === true;
  }

  if (kind === "exec.safe") {
    return value.length > 0 && ceiling.exec === true;
  }

  if (kind === "time.now") {
    return value.length > 0 && ceiling.time === true;
  }

  if (kind === "random.bytes") {
    return value.length > 0 && ceiling.random === true;
  }

  if (kind === "env") {
    return isValidEnvName(value) && Array.isArray(ceiling.env) && ceiling.env.includes(value);
  }

  if (kind === "net.http") {
    if (!Array.isArray(ceiling.net)) return false;
    let requested;
    try {
      requested = new URL(value);
    } catch {
      return false;
    }
    return ceiling.net.some((allowed) => {
      try {
        return netUriWithinPrefix(requested, new URL(allowed));
      } catch {
        return false;
      }
    });
  }

  if (kind === "fs.read" || kind === "fs.write") {
    const fsCaps = ceiling.fs ?? {};
    const key = kind.split(".")[1];
    if (!Array.isArray(fsCaps[key])) return false;
    const normalizedValue = normalizeFsPath(value);
    if (!normalizedValue) return false;
    return fsCaps[key].some((allowedPrefix) => {
      const normalizedPrefix = normalizeFsPath(allowedPrefix);
      if (!normalizedPrefix) return false;
      return isWithinPrefix(normalizedValue, normalizedPrefix);
    });
  }

  if (kind === "kv.read" || kind === "kv.write") {
    const kvCaps = ceiling.kv ?? {};
    const key = kind.split(".")[1];
    if (!Array.isArray(kvCaps[key])) return false;
    return kvCaps[key].includes("*") || kvCaps[key].includes(value);
  }

  if (kind === "queue.publish" || kind === "queue.consume") {
    const queueCaps = ceiling.queue ?? {};
    const key = kind.split(".")[1];
    if (!Array.isArray(queueCaps[key])) return false;
    return queueCaps[key].includes("*") || queueCaps[key].includes(value);
  }

  return false;
}

function runCapabilityVectors() {
  const schema = loadDoc("test-vectors/capability-eval/schema.json");
  const validate = ajv.compile(schema);
  const vectors = listFiles("test-vectors/capability-eval").filter((f) => !f.endsWith("schema.json"));
  let checks = 0;

  for (const vectorFile of vectors) {
    const vector = loadDoc(vectorFile);
    checks += 1;
    if (!validate(vector)) {
      throw new Error(`invalid vector schema: ${vectorFile}: ${ajv.errorsText(validate.errors)}`);
    }

    for (const testCase of vector.cases) {
      checks += 1;
      const decision = evaluateCapability(vector.policy, testCase.capability) ? "allow" : "deny";
      if (decision !== testCase.expect) {
        throw new Error(
          `capability mismatch in ${vectorFile} for ${testCase.capability.kind}:${testCase.capability.value}; expected=${testCase.expect} actual=${decision}`
        );
      }
    }

    console.log(`OK capability-eval ${vectorFile} cases=${vector.cases.length}`);
  }

  return checks;
}

const schemaGroups = [
  {
    schemaFile: "spec/v0/skill-manifest.schema.json",
    goodDir: "test-vectors/v0/skill-manifest/good",
    badDir: "test-vectors/v0/skill-manifest/bad"
  },
  {
    schemaFile: "spec/v0/pipeline.schema.json",
    goodDir: "test-vectors/v0/pipeline/good",
    badDir: "test-vectors/v0/pipeline/bad",
    semanticBad: ["test-vectors/v0/pipeline/bad/cycle.json"]
  },
  {
    schemaFile: "spec/policy/policy.schema.json",
    goodDir: "test-vectors/policy/valid",
    badDir: "test-vectors/policy/invalid"
  },
  {
    schemaFile: "spec/execution-receipt.schema.json",
    goodDir: "test-vectors/receipt/good",
    badDir: "test-vectors/receipt/bad",
    semanticBad: ["test-vectors/receipt/bad/hash-mismatch.json"]
  },
  {
    schemaFile: "spec/execution-receipt.v1.experimental.schema.json",
    goodDir: "test-vectors/receipt-v1/good",
    badDir: "test-vectors/receipt-v1/bad"
  },
  {
    schemaFile: "spec/registry/snapshot.schema.json",
    goodDir: "test-vectors/registry/snapshot/good",
    badDir: "test-vectors/registry/snapshot/bad",
    semanticBad: ["test-vectors/registry/snapshot/bad/hash-mismatch.json"]
  },
  {
    schemaFile: "spec/skill-format/manifest.schema.json",
    goodDir: "test-vectors/skill-format/manifest/good",
    badDir: "test-vectors/skill-format/manifest/bad"
  },
  {
    schemaFile: "spec/skill-format/provenance.schema.json",
    goodDir: "test-vectors/skill-format/provenance/good",
    badDir: "test-vectors/skill-format/provenance/bad"
  },
  {
    schemaFile: "spec/skill-format/signatures.schema.json",
    goodDir: "test-vectors/skill-format/signatures/good",
    badDir: "test-vectors/skill-format/signatures/bad"
  },
  {
    schemaFile: "spec/skill-format/manifest.v1.experimental.schema.json",
    goodDir: "test-vectors/skill-format/manifest-v1/good",
    badDir: "test-vectors/skill-format/manifest-v1/bad"
  }
];

let checks = 0;
for (const group of schemaGroups) {
  checks += assertSchemaGroup(group);
}
checks += runCapabilityVectors();

console.log(`OK conformance checks=${checks}`);
