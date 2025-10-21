import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = dirname(dirname(dirname(__dirname)));
const fixturesRoot = join(repoRoot, 'tests', 'fixtures', 'node');
const updateTranscripts = process.argv.includes('--update-transcripts');

async function loadFixture(transport, kind) {
  const path = join(fixturesRoot, transport, `${kind}.json`);
  try {
    const raw = await readFile(path, 'utf8');
    return JSON.parse(raw);
  } catch (error) {
    if (error && error.code === 'ENOENT') {
      throw Object.assign(new Error('fixture-missing'), { code: 'FIXTURE_MISSING' });
    }
    throw error;
  }
}

for (const transport of ['stdio', 'http', 'tls']) {
  test(`node client ${transport} transcript scaffolding`, async (t) => {
    let requestPayload;
    let responsePayload;
    try {
      requestPayload = await loadFixture(transport, 'request');
      responsePayload = await loadFixture(transport, 'response');
    } catch (error) {
      if (error.code === 'FIXTURE_MISSING') {
        t.skip('download the GitHub Action artifact to populate tests/fixtures/node');
        return;
      }
      throw error;
    }

    assert.equal(requestPayload.client, 'node');
    assert.equal(responsePayload.transport, transport);

    const cliCommand = [
      process.execPath,
      join(repoRoot, 'clients', 'node', 'index.mjs'),
      '--transport',
      transport,
      '--record-transcript',
      join(repoRoot, 'artifacts', 'node', `${transport}.json`),
    ];

    if (updateTranscripts) {
      cliCommand.push('--update-transcripts');
    }

    const result = spawnSync(cliCommand[0], cliCommand.slice(1), {
      cwd: repoRoot,
      stdio: 'pipe',
      encoding: 'utf8',
    });

    assert.notEqual(result.status, 0, 'placeholder command should fail until implemented');

    assert.fail('not yet implemented: node client subprocess invocation and transcript diffing');
  });
}
