'use strict';

const fs = require('node:fs/promises');
const fsSync = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawn } = require('node:child_process');
const ASR_MANIFEST = require('./offline-asr-engine.json');

const MAX_AUDIO_BYTES = 256 * 1024 * 1024;
const MAX_MODEL_BYTES = 4 * 1024 * 1024 * 1024;
const TRANSCRIBE_TIMEOUT_MS = 10 * 60 * 1000;
const MODEL_DOWNLOAD_TIMEOUT_MS = 30 * 60 * 1000;
const AUDIO_EXTENSIONS = new Set(['.wav', '.mp3', '.m4a', '.mp4', '.ogg', '.flac', '.webm', '.aac']);

function cleanString(value, limit = 4096) {
  return String(value ?? '').replace(/\0/g, '').trim().slice(0, limit);
}

function sha256File(filePath) {
  return new Promise((resolve, reject) => {
    const hash = crypto.createHash('sha256');
    const stream = fsSync.createReadStream(filePath);
    stream.on('error', reject);
    stream.on('data', (chunk) => hash.update(chunk));
    stream.on('end', () => resolve(hash.digest('hex')));
  });
}

async function fileExists(filePath) {
  try {
    const stat = await fs.stat(filePath);
    return stat.isFile();
  } catch {
    return false;
  }
}

function executableName() {
  return process.platform === 'win32' ? 'whisper-cli.exe' : 'whisper-cli';
}

function bundledBinaryCandidates(resourcesPath) {
  const executable = executableName();
  return [
    path.join(resourcesPath, 'asr', `${process.platform}-${process.arch}`, executable),
    path.join(resourcesPath, 'asr', executable),
  ];
}

function configuredBinary(resourcesPath) {
  const explicit = cleanString(process.env.FABUSHI_ASR_BINARY, 4096);
  return explicit ? path.resolve(explicit) : bundledBinaryCandidates(resourcesPath).find((candidate) => fsSync.existsSync(candidate)) ?? null;
}

function modelConfig(app, config = {}) {
  const explicitPath = cleanString(process.env.FABUSHI_ASR_MODEL_PATH, 4096);
  const modelDir = path.join(app.getPath('userData'), 'models', 'asr');
  const configuredUrl = cleanString(config.modelUrl ?? process.env.FABUSHI_ASR_MODEL_URL ?? (explicitPath ? null : ASR_MANIFEST.defaultModel?.url), 4096);
  const configuredSha = cleanString(config.sha256 ?? process.env.FABUSHI_ASR_MODEL_SHA256 ?? (explicitPath ? null : ASR_MANIFEST.defaultModel?.sha256), 128).toLowerCase();
  const filename = configuredUrl
    ? path.basename(new URL(configuredUrl).pathname) || 'model.bin'
    : 'model.bin';
  return {
    modelDir,
    modelPath: explicitPath ? path.resolve(explicitPath) : path.join(modelDir, filename),
    modelUrl: configuredUrl || null,
    sha256: /^[0-9a-f]{64}$/.test(configuredSha) ? configuredSha : null,
  };
}

async function inspectOfflineAsr({ app, resourcesPath, config = {} }) {
  const binaryPath = configuredBinary(resourcesPath);
  const model = modelConfig(app, config);
  const [binaryReady, modelReady] = await Promise.all([
    binaryPath ? fileExists(binaryPath) : Promise.resolve(false),
    fileExists(model.modelPath),
  ]);
  let modelVerified = modelReady && !model.sha256;
  let modelSha256 = null;
  if (modelReady && model.sha256) {
    modelSha256 = await sha256File(model.modelPath);
    modelVerified = modelSha256 === model.sha256;
  }
  return {
    available: Boolean(binaryReady && modelReady && modelVerified),
    engine: 'whisper.cpp',
    binaryPath: binaryReady ? binaryPath : null,
    modelPath: modelReady ? model.modelPath : null,
    modelUrlConfigured: Boolean(model.modelUrl),
    modelUrl: model.modelUrl,
    expectedSha256: model.sha256,
    expectedSizeBytes: Number(ASR_MANIFEST.defaultModel?.sizeBytes ?? 0) || null,
    modelSha256,
    modelVerified,
    missing: [
      ...(binaryReady ? [] : ['binary']),
      ...(modelReady ? [] : ['model']),
      ...(modelReady && !modelVerified ? ['model-integrity'] : []),
    ],
  };
}

async function downloadOfflineAsrModel({ app, net, resourcesPath, config = {}, onProgress }) {
  const model = modelConfig(app, config);
  if (!model.modelUrl) throw new Error('No offline ASR model URL is configured.');
  if (!model.sha256) throw new Error('Offline ASR model download requires a SHA-256 digest.');
  const url = new URL(model.modelUrl);
  if (url.protocol !== 'https:') throw new Error('Offline ASR model downloads require HTTPS.');
  await fs.mkdir(model.modelDir, { recursive: true, mode: 0o700 });
  const tempPath = `${model.modelPath}.download-${process.pid}-${Date.now()}`;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), MODEL_DOWNLOAD_TIMEOUT_MS);
  try {
    const response = await net.fetch(url.toString(), { signal: controller.signal });
    if (!response.ok || !response.body) throw new Error(`ASR model download failed with HTTP ${response.status}.`);
    const declared = Number(response.headers.get('content-length') ?? 0);
    if (declared > MAX_MODEL_BYTES) throw new Error('ASR model exceeds the maximum supported size.');
    const reader = response.body.getReader();
    const handle = await fs.open(tempPath, 'w', 0o600);
    let written = 0;
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        written += value.byteLength;
        if (written > MAX_MODEL_BYTES) throw new Error('ASR model exceeds the maximum supported size.');
        await handle.write(value);
        onProgress?.({ downloadedBytes: written, totalBytes: declared || null });
      }
    } finally {
      await handle.close();
    }
    if (model.sha256) {
      const digest = await sha256File(tempPath);
      if (digest !== model.sha256) throw new Error('ASR model SHA-256 verification failed.');
    }
    await fs.rename(tempPath, model.modelPath);
    return inspectOfflineAsr({ app, resourcesPath, config });
  } catch (error) {
    await fs.rm(tempPath, { force: true }).catch(() => undefined);
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

function normalizeLanguage(value) {
  const language = cleanString(value, 32).toLowerCase();
  if (!language || language === 'auto') return 'auto';
  return /^[a-z]{2,3}(?:-[a-z0-9]{2,8})?$/.test(language) ? language : 'auto';
}

async function transcribeOfflineAudio({ app, resourcesPath, config = {}, params }) {
  const status = await inspectOfflineAsr({ app, resourcesPath, config });
  if (!status.available) return { available: false, provider: 'fabushi-offline-asr', status };
  const inputPath = path.resolve(cleanString(params.path ?? params.filePath, 4096));
  const extension = path.extname(inputPath).toLowerCase();
  if (!AUDIO_EXTENSIONS.has(extension)) throw new Error(`Unsupported audio format: ${extension || 'unknown'}`);
  const stat = await fs.stat(inputPath);
  if (!stat.isFile()) throw new Error('Audio input is not a file.');
  if (stat.size <= 0 || stat.size > MAX_AUDIO_BYTES) throw new Error('Audio input is empty or exceeds the size limit.');
  const tempDir = await fs.mkdtemp(path.join(app.getPath('temp'), 'fabushi-asr-'));
  const outputPrefix = path.join(tempDir, 'transcript');
  const language = normalizeLanguage(params.language);
  const args = [
    '-m', status.modelPath,
    '-f', inputPath,
    '-otxt',
    '-of', outputPrefix,
    '-np',
  ];
  if (language !== 'auto') args.push('-l', language);
  if (params.translate === true) args.push('-tr');
  const startedAtMs = Date.now();
  try {
    const result = await new Promise((resolve, reject) => {
      const child = spawn(status.binaryPath, args, {
        shell: false,
        windowsHide: true,
        stdio: ['ignore', 'pipe', 'pipe'],
        env: { ...process.env, LC_ALL: 'C.UTF-8' },
      });
      let stderr = '';
      let stdout = '';
      const timer = setTimeout(() => {
        child.kill('SIGKILL');
        reject(new Error('Offline ASR transcription timed out.'));
      }, TRANSCRIBE_TIMEOUT_MS);
      child.stdout.on('data', (chunk) => { stdout = `${stdout}${String(chunk)}`.slice(-200_000); });
      child.stderr.on('data', (chunk) => { stderr = `${stderr}${String(chunk)}`.slice(-200_000); });
      child.once('error', (error) => { clearTimeout(timer); reject(error); });
      child.once('exit', (code, signal) => {
        clearTimeout(timer);
        if (code === 0) resolve({ stdout, stderr });
        else reject(new Error(`Offline ASR exited with ${code ?? signal ?? 'unknown'}: ${stderr.slice(-2000)}`));
      });
    });
    let text = '';
    const transcriptPath = `${outputPrefix}.txt`;
    if (await fileExists(transcriptPath)) text = (await fs.readFile(transcriptPath, 'utf8')).trim();
    if (!text) text = result.stdout.trim();
    return {
      available: true,
      provider: 'fabushi-offline-asr',
      engine: status.engine,
      language,
      text,
      durationMs: Date.now() - startedAtMs,
      modelPath: status.modelPath,
    };
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true }).catch(() => undefined);
  }
}

module.exports = {
  inspectOfflineAsr,
  downloadOfflineAsrModel,
  transcribeOfflineAudio,
};
