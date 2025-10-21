import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { access } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';
import test from 'node:test';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');
const repositoryRoot = path.resolve(projectRoot, '..', '..');
const fixturesRoot = path.resolve(repositoryRoot, 'tests', 'fixtures', 'node');
const transports = ['stdio', 'http', 'tls'];
const updateTranscripts = process.argv.includes('--update-transcripts');

async function loadFixture(transport, kind) {
  const filePath = path.join(fixturesRoot, transport, `${kind}.json`);
  const data = await readFile(filePath, 'utf8');
  return JSON.parse(data);
}

async function runClientSubprocess(cliEntry, transport) {
  await access(cliEntry);

  return new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [cliEntry, '--transport', transport], {
      stdio: 'pipe',
      env: process.env,
    });

    const output = { stdout: '', stderr: '' };
    child.stdout?.setEncoding('utf8');
    child.stderr?.setEncoding('utf8');
    child.stdout?.on('data', (chunk) => {
      output.stdout += chunk;
    });
    child.stderr?.on('data', (chunk) => {
      output.stderr += chunk;
    });

    child.once('error', (error) => {
      reject(error);
    });

    child.once('exit', (code) => {
      reject(new Error(`process exited with code ${code}\nSTDOUT:\n${output.stdout}\nSTDERR:\n${output.stderr}`));
    });
  });
}

for (const transport of transports) {
  test(`node client ${transport} transport fixtures`, async (t) => {
    const request = await loadFixture(transport, 'request');
    const response = await loadFixture(transport, 'response');

    assert.equal(request.params.transport, transport);
    assert.equal(response.id, request.id);

    if (updateTranscripts) {
      t.skip('transcript regeneration not yet implemented for Node client tests');
      return;
    }

    const cliEntry = path.resolve(projectRoot, 'index.mjs');

    try {
      await runClientSubprocess(cliEntry, transport);
    } catch (error) {
      assert.fail(
        `not yet implemented: Node client CLI execution placeholder (${error.message})` +
          `\nCommand: node ${cliEntry} --transport ${transport}`,
      );
    }

    assert.fail('not yet implemented: Node client subprocess execution should assert transcripts');
  });
}
