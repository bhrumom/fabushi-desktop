use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedImage {
    pub image_data: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedImage {
    pub absolute_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SandGenerateImageError {
    #[error("{0}")]
    Generate(String),
    #[error("Failed to save the generated image into the agent's media store.")]
    Persist,
    #[error("Generated image data was not valid base64.")]
    InvalidBase64,
}

pub type GenerateImageBackend =
    Arc<dyn Fn(&str, &[(String, String)]) -> Result<GeneratedImage, String> + Send + Sync>;
pub type PersistGeneratedImage =
    Arc<dyn Fn(&[u8], &str) -> Result<Option<PersistedImage>, String> + Send + Sync>;

#[derive(Clone)]
pub struct SandGenerateImageService {
    generate: GenerateImageBackend,
    persist: PersistGeneratedImage,
}

impl SandGenerateImageService {
    pub fn new(generate: GenerateImageBackend, persist: PersistGeneratedImage) -> Self {
        Self { generate, persist }
    }

    pub fn generate(
        &self,
        description: &str,
        reference_images: &[(String, String)],
    ) -> Result<(String, String), SandGenerateImageError> {
        let generated = (self.generate)(description, reference_images)
            .map_err(SandGenerateImageError::Generate)?;
        let bytes = STANDARD
            .decode(generated.image_data.as_bytes())
            .map_err(|_| SandGenerateImageError::InvalidBase64)?;
        let persisted = (self.persist)(&bytes, &generated.mime_type)
            .map_err(SandGenerateImageError::Generate)?
            .ok_or(SandGenerateImageError::Persist)?;
        Ok((persisted.absolute_path, generated.image_data))
    }
}
