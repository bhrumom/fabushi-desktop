use std::sync::Arc;

use crate::extensions::attachments::generate_image_service::{
    GenerateImageAuth, PersistGeneratedImage, SandGenerateImageService,
};
use crate::extensions::inference::provider_session::ProviderSessionError;
use crate::runner::tools::sand_generate_image_tool::{
    GenerateImageToolExecutor, GeneratedImageToolOutput,
};

pub struct ProductionGenerateImageToolExecutor {
    auth: Arc<dyn GenerateImageAuth>,
    persist: PersistGeneratedImage,
}

impl ProductionGenerateImageToolExecutor {
    pub fn new(
        auth: Arc<dyn GenerateImageAuth>,
        persist: PersistGeneratedImage,
    ) -> Self {
        Self { auth, persist }
    }
}

impl GenerateImageToolExecutor for ProductionGenerateImageToolExecutor {
    fn generate(
        &self,
        description: &str,
        reference_images: &[(String, String)],
    ) -> Result<GeneratedImageToolOutput, ProviderSessionError> {
        let service = SandGenerateImageService::production(
            Arc::clone(&self.auth),
            Arc::clone(&self.persist),
        )
        .map_err(ProviderSessionError::Tool)?;
        let (persisted_path, image_data_base64) = service
            .generate(description, reference_images)
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        Ok(GeneratedImageToolOutput {
            persisted_path,
            image_data_base64,
        })
    }
}
