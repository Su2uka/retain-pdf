// vendor/cordis 里的第三方代码必须和上游一致，只允许 README 里写明的那一处改动。
import assert from "node:assert/strict";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const vendorDir = path.join(path.dirname(fileURLToPath(import.meta.url)), "vendor", "cordis");
const sha256 = (text) => crypto.createHash("sha256").update(text).digest("hex");

test("cosmokit.mjs is byte-identical to cosmokit@1.8.1 lib/index.mjs", () => {
  const text = fs.readFileSync(path.join(vendorDir, "cosmokit.mjs"), "utf8");
  assert.equal(sha256(text), "8c493c6ab7e44ab7608e407e84a79e86dfb6c17b8067cbccfcf29c011285f72a");
});

test("cordis.mjs equals cordis@4.0.0-rc.12 lib/index.js except the cosmokit import path", () => {
  const text = fs.readFileSync(path.join(vendorDir, "cordis.mjs"), "utf8");
  const imports = [...text.matchAll(/^import .* from "([^"]+)";$/gm)].map((match) => match[1]);
  assert.deepEqual([...new Set(imports)], ["./cosmokit.mjs"], "the only import is the vendored cosmokit");
  const upstream = text.replaceAll('from "./cosmokit.mjs";', 'from "cosmokit";');
  assert.equal(sha256(upstream), "cabbe279876480eddd84e344c2fdc8e7e5eb32580a4e1ea24b0e05a46ad0c2ed");
});
