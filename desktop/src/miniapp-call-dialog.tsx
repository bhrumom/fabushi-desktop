import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type {
  MiniAppBotCallProgram,
  MiniAppBotCallRoute,
  MiniAppBotCallState,
} from './miniapp-bot-projection';

const CALL_PROTOCOL = 'fabushi.miniapp.call.v1';
const MAX_RECORDING_BYTES = 256 * 1024 * 1024;

type MiniAppCallDialogProps = {
  callId: string;
  miniAppId: string;
  title: string;
  kind: 'voice' | 'video';
  program: MiniAppBotCallProgram;
  html?: string;
  onCommand: (command: string, args?: Record<string, unknown>) => Promise<void>;
  onNaturalLanguage?: (input: string) => Promise<void>;
  onSaveRecording: (blob: Blob, mimeType: string) => Promise<void>;
  onClose: () => void;
};

type CallFrameMessage = {
  protocol?: string;
  action?: string;
  miniAppId?: string;
  callId?: string;
};

function recorderMimeType(): string | undefined {
  if (typeof MediaRecorder === 'undefined') return undefined;
  const candidates = [
    'video/webm;codecs=vp9,opus',
    'video/webm;codecs=vp8,opus',
    'video/webm',
  ];
  return candidates.find((candidate) => MediaRecorder.isTypeSupported(candidate));
}

function findState(program: MiniAppBotCallProgram, stateId?: string): MiniAppBotCallState | undefined {
  return program.states.find((state) => state.id === stateId);
}

function postToFrame(
  frame: HTMLIFrameElement | null,
  payload: Record<string, unknown>,
): void {
  frame?.contentWindow?.postMessage(payload, '*');
}

export function MiniAppCallDialog({
  callId,
  miniAppId,
  title,
  kind,
  program,
  html,
  onCommand,
  onNaturalLanguage,
  onSaveRecording,
  onClose,
}: MiniAppCallDialogProps) {
  const frameRef = useRef<HTMLIFrameElement | null>(null);
  const videoRef = useRef<HTMLVideoElement | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  const recordingBytesRef = useRef(0);
  const closingAfterRecordingRef = useRef(false);
  const stateHistoryRef = useRef<string[]>([]);
  const [mediaReady, setMediaReady] = useState(false);
  const [recording, setRecording] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentStateId, setCurrentStateId] = useState(program.startState ?? program.states[0]?.id ?? '');
  const [naturalLanguage, setNaturalLanguage] = useState('');
  const [ivrBusy, setIvrBusy] = useState(false);

  const currentState = useMemo(
    () => findState(program, currentStateId) ?? program.states[0],
    [currentStateId, program],
  );

  const stopTracks = useCallback(() => {
    const stream = streamRef.current;
    streamRef.current = null;
    stream?.getTracks().forEach((track) => track.stop());
    if (videoRef.current) videoRef.current.srcObject = null;
    setMediaReady(false);
  }, []);

  const sendFrame = useCallback((action: string, payload: Record<string, unknown> = {}) => {
    postToFrame(frameRef.current, {
      protocol: CALL_PROTOCOL,
      action,
      miniAppId,
      callId,
      kind,
      ...payload,
    });
  }, [callId, kind, miniAppId]);

  const finishClose = useCallback(() => {
    stopTracks();
    onClose();
  }, [onClose, stopTracks]);

  const stopRecording = useCallback(() => {
    const recorder = recorderRef.current;
    if (!recorder || recorder.state === 'inactive') return;
    recorder.stop();
  }, []);

  const requestClose = useCallback(() => {
    if (recording && recorderRef.current?.state !== 'inactive') {
      closingAfterRecordingRef.current = true;
      stopRecording();
      return;
    }
    finishClose();
  }, [finishClose, recording, stopRecording]);

  const startRecording = useCallback(() => {
    if (program.type !== 'miniapp-surface' || kind !== 'video') return;
    if (!streamRef.current) {
      const message = '摄像头和麦克风尚未就绪。';
      setError(message);
      sendFrame('record.error', { error: message });
      return;
    }
    if (typeof MediaRecorder === 'undefined') {
      const message = '当前运行环境不支持视频录制。';
      setError(message);
      sendFrame('record.error', { error: message });
      return;
    }
    if (recorderRef.current && recorderRef.current.state !== 'inactive') return;

    const mimeType = recorderMimeType();
    let recorder: MediaRecorder;
    try {
      recorder = mimeType
        ? new MediaRecorder(streamRef.current, { mimeType })
        : new MediaRecorder(streamRef.current);
    } catch (recordError) {
      const message = recordError instanceof Error ? recordError.message : '无法启动视频录制。';
      setError(message);
      sendFrame('record.error', { error: message });
      return;
    }

    chunksRef.current = [];
    recordingBytesRef.current = 0;
    recorderRef.current = recorder;
    recorder.ondataavailable = (event) => {
      if (!event.data.size) return;
      recordingBytesRef.current += event.data.size;
      if (recordingBytesRef.current > MAX_RECORDING_BYTES) {
        const message = '本次录制已达到 256 MB 安全上限，已自动停止。';
        setError(message);
        sendFrame('record.error', { error: message });
        if (recorder.state !== 'inactive') recorder.stop();
        return;
      }
      chunksRef.current.push(event.data);
    };
    recorder.onerror = () => {
      const message = '录制过程中发生媒体错误。';
      setError(message);
      setRecording(false);
      sendFrame('record.error', { error: message });
    };
    recorder.onstop = () => {
      const blobs = chunksRef.current;
      chunksRef.current = [];
      const recordingMime = recorder.mimeType || mimeType || 'video/webm';
      const blob = new Blob(blobs, { type: recordingMime });
      setRecording(false);
      if (!blob.size) {
        const message = '没有产生可保存的视频数据。';
        setError(message);
        sendFrame('record.error', { error: message });
        if (closingAfterRecordingRef.current) {
          closingAfterRecordingRef.current = false;
          finishClose();
        }
        return;
      }
      setSaving(true);
      void onSaveRecording(blob, recordingMime)
        .then(() => {
          setError(null);
          sendFrame('record.saved', { size: blob.size, mimeType: recordingMime });
        })
        .catch((saveError) => {
          const message = saveError instanceof Error ? saveError.message : '保存录制视频失败。';
          setError(message);
          sendFrame('record.error', { error: message });
        })
        .finally(() => {
          setSaving(false);
          if (closingAfterRecordingRef.current) {
            closingAfterRecordingRef.current = false;
            finishClose();
          }
        });
    };

    try {
      recorder.start(1000);
      setError(null);
      setRecording(true);
      sendFrame('record.started', { mimeType: recorder.mimeType || mimeType || 'video/webm' });
    } catch (recordError) {
      recorderRef.current = null;
      const message = recordError instanceof Error ? recordError.message : '无法启动视频录制。';
      setError(message);
      sendFrame('record.error', { error: message });
    }
  }, [finishClose, kind, onSaveRecording, program.type, sendFrame]);

  useEffect(() => {
    if (program.type !== 'miniapp-surface') return undefined;
    let cancelled = false;
    const constraints: MediaStreamConstraints = kind === 'video'
      ? { audio: true, video: { facingMode: 'user' } }
      : { audio: true, video: false };

    if (!navigator.mediaDevices?.getUserMedia) {
      const message = '当前运行环境无法访问摄像头或麦克风。';
      setError(message);
      sendFrame('media.error', { error: message });
      return undefined;
    }

    void navigator.mediaDevices.getUserMedia(constraints)
      .then((stream) => {
        if (cancelled) {
          stream.getTracks().forEach((track) => track.stop());
          return;
        }
        streamRef.current = stream;
        if (videoRef.current && kind === 'video') {
          videoRef.current.srcObject = stream;
          void videoRef.current.play().catch(() => undefined);
        }
        setMediaReady(true);
        setError(null);
        sendFrame('media.ready');
      })
      .catch((mediaError) => {
        const message = mediaError instanceof Error
          ? `无法使用摄像头或麦克风：${mediaError.message}`
          : '无法使用摄像头或麦克风，请检查权限。';
        setError(message);
        sendFrame('media.error', { error: message });
      });

    return () => {
      cancelled = true;
      const recorder = recorderRef.current;
      recorderRef.current = null;
      if (recorder && recorder.state !== 'inactive') {
        recorder.ondataavailable = null;
        recorder.onstop = null;
        recorder.onerror = null;
        recorder.stop();
      }
      stopTracks();
    };
  }, [kind, program.type, sendFrame, stopTracks]);

  useEffect(() => {
    if (program.type !== 'service-call' || !currentState?.prompt) return;
    if (!('speechSynthesis' in window) || typeof SpeechSynthesisUtterance === 'undefined') return;
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(currentState.prompt);
    utterance.lang = 'zh-CN';
    window.speechSynthesis.speak(utterance);
    return () => window.speechSynthesis.cancel();
  }, [currentState?.prompt, program.type]);

  const applyRoute = useCallback(async (route: MiniAppBotCallRoute) => {
    if (ivrBusy) return;
    setIvrBusy(true);
    setError(null);
    try {
      if (route.action === 'end') {
        requestClose();
        return;
      }
      if (route.action === 'back') {
        const previous = stateHistoryRef.current.pop();
        if (previous) setCurrentStateId(previous);
        return;
      }
      if (route.action === 'command') {
        if (!route.command) throw new Error('这个 IVR 选项没有声明 Mini App 命令。');
        await onCommand(route.command, route.arguments);
      }
      if (route.nextState) {
        if (currentStateId) stateHistoryRef.current.push(currentStateId);
        setCurrentStateId(route.nextState);
      }
    } catch (routeError) {
      setError(routeError instanceof Error ? routeError.message : '服务执行失败，请重试。');
    } finally {
      setIvrBusy(false);
    }
  }, [currentStateId, ivrBusy, onCommand, requestClose]);

  const submitNaturalLanguage = useCallback(async () => {
    const input = naturalLanguage.trim();
    if (!input || !onNaturalLanguage || ivrBusy) return;
    setIvrBusy(true);
    setError(null);
    try {
      await onNaturalLanguage(input);
      setNaturalLanguage('');
    } catch (naturalError) {
      setError(naturalError instanceof Error ? naturalError.message : 'AI 语音/自然语言服务暂不可用，请继续使用下方固定菜单。');
    } finally {
      setIvrBusy(false);
    }
  }, [ivrBusy, naturalLanguage, onNaturalLanguage]);

  useEffect(() => {
    if (program.type !== 'miniapp-surface') return undefined;
    const onMessage = (event: MessageEvent) => {
      if (event.source !== frameRef.current?.contentWindow) return;
      const data = event.data as CallFrameMessage | null;
      if (!data || data.protocol !== CALL_PROTOCOL || data.miniAppId !== miniAppId || data.callId !== callId) return;
      if (data.action === 'ready') {
        sendFrame('call.start', { mediaReady });
        if (mediaReady) sendFrame('media.ready');
        return;
      }
      if (data.action === 'record.start') {
        startRecording();
        return;
      }
      if (data.action === 'record.stop') {
        stopRecording();
        return;
      }
      if (data.action === 'close') requestClose();
    };
    window.addEventListener('message', onMessage);
    return () => window.removeEventListener('message', onMessage);
  }, [callId, mediaReady, miniAppId, program.type, requestClose, sendFrame, startRecording, stopRecording]);

  const rootStyle: React.CSSProperties = {
    position: 'fixed', inset: 0, zIndex: 10000, display: 'grid', placeItems: 'center',
    background: 'rgba(5,8,14,.82)', backdropFilter: 'blur(16px)', color: '#f7f8fc',
  };
  const panelStyle: React.CSSProperties = {
    position: 'relative', width: program.type === 'miniapp-surface' ? 'min(1180px, 96vw)' : 'min(560px, 92vw)',
    height: program.type === 'miniapp-surface' ? 'min(820px, 94vh)' : 'auto', maxHeight: '94vh', overflow: 'hidden',
    borderRadius: 24, border: '1px solid rgba(255,255,255,.15)', background: '#111722', boxShadow: '0 30px 100px rgba(0,0,0,.5)',
  };

  if (program.type === 'miniapp-surface') {
    return (
      <div style={rootStyle} role="dialog" aria-modal="true" aria-label={`${title} ${kind === 'video' ? '视频' : '语音'}通话`}>
        <div style={panelStyle}>
          {kind === 'video' ? (
            <video
              ref={videoRef}
              muted
              playsInline
              autoPlay
              style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'cover', transform: 'scaleX(-1)', background: '#05070b' }}
            />
          ) : <div style={{ position: 'absolute', inset: 0, background: 'radial-gradient(circle at 50% 35%, #26334a, #090d14 65%)' }} />}
          {html ? (
            <iframe
              ref={frameRef}
              title={`${title} 通话界面`}
              srcDoc={html}
              sandbox="allow-scripts allow-forms"
              style={{ position: 'absolute', inset: 0, width: '100%', height: '100%', border: 0, background: 'transparent' }}
              onLoad={() => sendFrame('call.start', { mediaReady })}
            />
          ) : null}
          <div style={{ position: 'absolute', top: 18, left: 18, zIndex: 3, pointerEvents: 'none', padding: '8px 12px', borderRadius: 999, background: 'rgba(8,11,17,.68)', border: '1px solid rgba(255,255,255,.14)', fontSize: 13 }}>
            {saving ? '正在保存视频…' : recording ? '● 正在录制' : mediaReady ? program.title : '正在准备媒体…'}
          </div>
          <button
            type="button"
            onClick={requestClose}
            disabled={saving}
            aria-label="结束通话"
            style={{ position: 'absolute', zIndex: 4, top: 18, right: 18, width: 44, height: 44, borderRadius: '50%', border: '1px solid rgba(255,255,255,.16)', background: 'rgba(10,12,18,.72)', color: '#fff', fontSize: 24 }}
          >×</button>
          {error ? (
            <div role="alert" style={{ position: 'absolute', zIndex: 4, right: 18, bottom: 18, maxWidth: 440, padding: '10px 13px', borderRadius: 12, background: 'rgba(138,26,39,.92)', fontSize: 13 }}>
              {error}
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  return (
    <div style={rootStyle} role="dialog" aria-modal="true" aria-label={`${title} 固定语音服务`}>
      <div style={{ ...panelStyle, padding: 22, overflow: 'auto' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: 12, marginBottom: 16 }}>
          <div>
            <div style={{ fontSize: 13, color: '#9eabc1' }}>{program.title}</div>
            <strong style={{ fontSize: 20 }}>{title}</strong>
          </div>
          <button type="button" onClick={requestClose} style={{ width: 40, height: 40, borderRadius: '50%', border: '1px solid #344056', background: '#1a2331', color: '#fff', fontSize: 22 }}>×</button>
        </div>
        <div style={{ padding: 16, borderRadius: 16, background: '#172131', border: '1px solid #2a3850', lineHeight: 1.7 }}>
          {currentState?.prompt ?? '这个 Mini App 没有可用的固定服务状态。'}
        </div>
        {program.aiMode === 'optional' && onNaturalLanguage ? (
          <div style={{ marginTop: 14, padding: 14, borderRadius: 16, border: '1px solid #29364a', background: '#101722' }}>
            <div style={{ marginBottom: 8, fontSize: 12, color: '#9eabc1' }}>AI 可用时可以直接描述需求；不可用时不影响下方固定流程。</div>
            <div style={{ display: 'flex', gap: 8 }}>
              <input
                value={naturalLanguage}
                onChange={(event) => setNaturalLanguage(event.target.value)}
                onKeyDown={(event) => { if (event.key === 'Enter') void submitNaturalLanguage(); }}
                placeholder="例如：帮我查询当前余额"
                style={{ flex: 1, minWidth: 0, border: '1px solid #36445b', borderRadius: 12, padding: '10px 12px', background: '#0b111a', color: '#fff' }}
              />
              <button type="button" onClick={() => void submitNaturalLanguage()} disabled={ivrBusy || !naturalLanguage.trim()} style={{ border: 0, borderRadius: 12, padding: '0 14px', background: '#f3f5f9', color: '#111722', fontWeight: 700 }}>发送</button>
            </div>
          </div>
        ) : null}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, minmax(0, 1fr))', gap: 10, marginTop: 16 }}>
          {(currentState?.routes ?? []).map((route) => (
            <button
              key={`${currentState?.id}:${route.digits}`}
              type="button"
              disabled={ivrBusy}
              onClick={() => void applyRoute(route)}
              style={{ minHeight: 66, borderRadius: 14, border: '1px solid #34435a', background: '#1a2636', color: '#fff', padding: 10 }}
            >
              <strong style={{ display: 'block', fontSize: 20 }}>{route.digits}</strong>
              <span style={{ display: 'block', marginTop: 4, fontSize: 11, color: '#aeb9cc' }}>{route.label ?? (route.action === 'end' ? '结束' : route.command ?? route.nextState ?? route.action)}</span>
            </button>
          ))}
        </div>
        <div style={{ marginTop: 14, fontSize: 12, color: '#8f9bb1' }}>
          {program.aiMode === 'disabled' ? '固定流程模式 · 不消耗 AI 额度' : '固定流程始终可用 · AI 是可选增强'}
        </div>
        {error ? <div role="alert" style={{ marginTop: 12, padding: 10, borderRadius: 10, background: '#5d1e2a', color: '#ffdfe4', fontSize: 12 }}>{error}</div> : null}
      </div>
    </div>
  );
}
