import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import path from 'node:path';
const root = path.resolve(import.meta.dirname, '../..');
const require = createRequire(import.meta.url);
const { studioOperationPostures, studioOperationPosture } = require(path.join(root, 'build/studio-kernel-test/studio/src/contrib/settings/capabilityPosture.js'));
const binary = process.env.YAI_STUDIO_TEST_BINARY ?? path.join(root, 'cmd/yai/target/debug/yai');
const catalog = JSON.parse(execFileSync(binary, ['capabilities','--json'], {encoding:'utf8'})).data.value;
assert.deepEqual(Object.keys(studioOperationPostures).sort(), catalog.operations.map(op=>op.operation_id).sort(), 'Every published operation needs a deliberate Studio posture; unreviewed delta must fail');
for (const entry of Object.values(studioOperationPostures)) {
 assert.ok(entry.detail);
 if (entry.state === 'integrated') assert.ok(existsSync(path.join(root, entry.proof)), entry.proof);
}
assert.equal(studioOperationPosture('future.unreviewed.operation').state,'debt');
console.log(JSON.stringify({result:'PASS',advertised:catalog.operations.length,integrated:Object.values(studioOperationPostures).filter(x=>x.state==='integrated').length,proof:'Catalog-derived exact identity coverage; referenced product suites must run separately, this inventory is not behavioral qualification'}));
