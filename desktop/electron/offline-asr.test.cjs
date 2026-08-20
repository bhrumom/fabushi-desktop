'use strict';

const assert = require('node:assert/strict');
const crypto = require('node:crypto');
const fs = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');

const {
  inspectOfflineAsr,
  downloadOfflineAsrModel,
  transcribeOfflineAudio,
} = require('./offline-asr.cjs');

async function withTemp(run) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'fabushi-asr-test-'));
  try {
    const app = {
      getPath(name) {
        if (name === 'userData') return path.join(root, 'user-data');
        if (name === 'temp') return path.join(root, 'temp');
        throw new Error(`unexpected app path: ${name}`);
      },
    };
    await fs.mkdir(app.getPath('userData'), { recursive: true });
    await fs.mkdir(app.getPath('temp'), { recursive: true });
    await run({ root, app });
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
}

async function fakeWhisperBinary(root) {
  const binary = path.join(root, 'whisper-cli');
  await fs.writeFile(binary, `#!/usr/bin/env node
const fs = require('node:fs');
const args = process.argv.slice(2);
const outIndex = args.indexOf('-of');
if (outIndex < 0 || !args[outIndex + 1]) process.exit(9);
fs.writeFileSync(args[outIndex + 1] + '.txt', 'local transcription works\\n', 'utf8');
`, { mode: 0o700 });
  await fs.chmod(binary, 0o700);
  return binary;
}

test('offline ASR invokes a local whisper-compatible binary without shell execution', async () => {
  await withTemp(async ({ root, app }) => {
    const binary = await fakeWhisperBinary(root);
    const model = path.join(root, 'model.bin');
    const audio = path.join(root, 'sample.wav');
    await fs.writeFile(model, Buffer.from('model'));
    await fs.writeFile(audio, Buffer.from('RIFF-test-audio'));
    const previousBinary = process.env.FABUSHI_ASR_BINARY;
    const previousModel = process.env.FABUSHI_ASR_MODEL_PATH;
    process.env.FABUSHI_ASR_BINARY = binary;
    process.env.FABUSHI_ASR_MODEL_PATH = model;
    try {
      const status = await inspectOfflineAsr({ app, resourcesPath: root });
      assert.equal(status.available, true);
      const result = await transcribeOfflineAudio({
        app,
        resourcesPath: root,
        params: { path: audio, language: 'en' },
      });
      assert.equal(result.available, true);
      assert.equal(result.provider, 'fabushi-offline-asr');
      assert.equal(result.text, 'local transcription works');
      assert.equal(result.language, 'en');
    } finally {
      if (previousBinary === undefined) delete process.env.FABUSHI_ASR_BINARY;
      else process.env.FABUSHI_ASR_BINARY = previousBinary;
      if (previousModel === undefined) delete process.env.FABUSHI_ASR_MODEL_PATH;
      else process.env.FABUSHI_ASR_MODEL_PATH = previousModel;
    }
  });
});

test('offline ASR downloads only HTTPS models whose SHA-256 matches', async () => {
  await withTemp(async ({ root, app }) => {
    const binary = await fakeWhisperBinary(root);
    const previousBinary = process.env.FABUSHI_ASR_BINARY;
    const previousModel = process.env.FABUSHI_ASR_MODEL_PATH;
    process.env.FABUSHI_ASR_BINARY = binary;
    delete process.env.FABUSHI_ASR_MODEL_PATH;
    const bytes = Buffer.from('verified-model-bytes');
    const sha256 = crypto.createHash('sha256').update(bytes).digest('hex');
    const net = {
      async fetch(url) {
        assert.equal(url, 'https://models.example.test/fabushi-model.bin');
        return new Response(bytes, {
          status: 200,
          headers: { 'content-length': String(bytes.length) },
        });
      },
    };
    try {
      const status = await downloadOfflineAsrModel({
        app,
        net,
        resourcesPath: root,
        config: {
          modelUrl: 'https://models.example.test/fabushi-model.bin',
          sha256,
        },
      });
      assert.equal(status.available, true);
      assert.equal(status.modelVerified, true);
      assert.equal(status.modelSha256, sha256);
    } finally {
      if (previousBinary === undefined) delete process.env.FABUSHI_ASR_BINARY;
      else process.env.FABUSHI_ASR_BINARY = previousBinary;
      if (previousModel === undefined) delete process.env.FABUSHI_ASR_MODEL_PATH;
      else process.env.FABUSHI_ASR_MODEL_PATH = previousModel;
    }
  });
});

test('offline ASR rejects model integrity mismatch and removes partial downloads', async () => {
  await withTemp(async ({ root, app }) => {
    const bytes = Buffer.from('tampered-model');
    const net = {
      async fetch() {
        return new Response(bytes, { status: 200 });
      },
    };
    await assert.rejects(
      downloadOfflineAsrModel({
        app,
        net,
        resourcesPath: root,
        config: {
          modelUrl: 'https://models.example.test/bad.bin',
          sha256: '0'.repeat(64),
        },
      }),
      /SHA-256 verification failed/,
    );
    const modelDir = path.join(app.getPath('userData'), 'models', 'asr');
    const entries = await fs.readdir(modelDir).catch(() => []);
    assert.deepEqual(entries, []);
  });
});
