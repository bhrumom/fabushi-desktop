use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceSessionPhase {
    Inactive,
    Listening,
    Thinking,
    Speaking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TalkbackMode {
    AgentOnly,
    Bidirectional,
}

#[derive(Clone)]
pub struct TalkSessionController {
    phase: VoiceSessionPhase,
    mode: TalkbackMode,
    silence_threshold_rms: f32,
}

impl Default for TalkSessionController {
    fn default() -> Self {
        Self {
            phase: VoiceSessionPhase::Inactive,
            mode: TalkbackMode::Bidirectional,
            silence_threshold_rms: 0.015,
        }
    }
}

impl TalkSessionController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn phase(&self) -> VoiceSessionPhase {
        self.phase
    }

    pub fn set_phase(&mut self, phase: VoiceSessionPhase) {
        self.phase = phase;
    }

    pub fn mode(&self) -> TalkbackMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: TalkbackMode) {
        self.mode = mode;
    }

    pub fn compute_audio_rms(pcm_bytes: &[u8]) -> f32 {
        if pcm_bytes.len() < 2 {
            return 0.0;
        }

        let mut sum_sq = 0.0;
        let mut sample_count = 0;

        for chunk in pcm_bytes.chunks_exact(2) {
            let sample = i16::from_le_bytes([chunk[0], chunk[1]]) as f32 / 32768.0;
            sum_sq += sample * sample;
            sample_count += 1;
        }

        if sample_count == 0 {
            0.0
        } else {
            (sum_sq / sample_count as f32).sqrt()
        }
    }

    pub fn is_speech_detected(&self, pcm_bytes: &[u8]) -> bool {
        Self::compute_audio_rms(pcm_bytes) > self.silence_threshold_rms
    }
}
