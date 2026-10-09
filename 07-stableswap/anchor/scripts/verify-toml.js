/**
 * Sanity check: confirm the installed `toml` package can still parse this
 * project's Anchor.toml the same way @anchor-lang/core does internally.
 *
 * Why this exists: `toml` is pinned to ^4.2.0 via the root `resolutions`
 * field (to pick up fixes for GHSA advisories not patched in any 3.x
 * release), even though @anchor-lang/core's own package.json still
 * declares "toml": "^3.0.0". Nothing else in this project's JS tooling
 * exercises `toml`, so a breaking change between 3.x and 4.x (or any
 * future bump) would otherwise go unnoticed until `anchor.workspace.*`
 * failed at runtime.
 *
 * This mirrors the exact access pattern used by
 * @anchor-lang/core/dist/cjs/workspace.js:
 *   const anchorToml = toml.parse(fs.readFileSync("Anchor.toml"));
 *   const clusterId = anchorToml.provider.cluster;
 *   const programs = anchorToml.programs?.[clusterId];
 *
 * Run with: node scripts/verify-toml.js
 * (also wired into `postinstall` so it runs automatically on install)
 */
const fs = require("fs");
const path = require("path");
const toml = require("toml");

const anchorTomlPath = path.join(__dirname, "..", "Anchor.toml");

let parsed;
try {
  const raw = fs.readFileSync(anchorTomlPath, "utf8");
  parsed = toml.parse(raw);
} catch (err) {
  console.error(
    `[verify-toml] Failed to parse ${anchorTomlPath} with toml@${
      require("toml/package.json").version
    }:`
  );
  console.error(err);
  process.exit(1);
}

const clusterId = parsed && parsed.provider && parsed.provider.cluster;
const programs = parsed && parsed.programs && parsed.programs[clusterId];

if (!clusterId || !programs) {
  console.error(
    `[verify-toml] Parsed Anchor.toml with toml@${
      require("toml/package.json").version
    } but it is missing expected [provider.cluster] / [programs.<cluster>] sections that @anchor-lang/core's workspace API relies on.`
  );
  console.error("Parsed result:", JSON.stringify(parsed, null, 2));
  process.exit(1);
}

console.log(
  `[verify-toml] OK: toml@${
    require("toml/package.json").version
  } parsed Anchor.toml correctly (provider.cluster="${clusterId}", programs.${clusterId}=${JSON.stringify(
    programs
  )}).`
);
