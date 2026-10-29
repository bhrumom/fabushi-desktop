use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerWarmImageGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerWarmImageState {
    Missing,
    Pending,
    Available,
    Retiring,
    CaptureUncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudWorkerWarmImageOperation {
    Allocate,
    Capture,
    Fork,
    Release,
    Retire,
    Delete,
    Recover,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloudWorkerWarmImage {
    pub image_id: String,
    pub generation: CloudWorkerWarmImageGeneration,
    pub state: CloudWorkerWarmImageState,
    pub hold_count: u32,
    pub cleanup_debt: bool,
}

impl CloudWorkerWarmImage {
    pub fn new(image_id: impl Into<String>) -> Result<Self, CloudWorkerWarmImageError> {
        let image_id = image_id.into();
        if image_id.trim().is_empty() {
            return Err(CloudWorkerWarmImageError::InvalidImageId);
        }
        Ok(Self {
            image_id,
            generation: CloudWorkerWarmImageGeneration(0),
            state: CloudWorkerWarmImageState::Missing,
            hold_count: 0,
            cleanup_debt: false,
        })
    }

    pub fn compare_and_apply(
        &mut self,
        expected_generation: CloudWorkerWarmImageGeneration,
        operation: CloudWorkerWarmImageOperation,
    ) -> Result<(), CloudWorkerWarmImageError> {
        if self.generation != expected_generation {
            return Err(CloudWorkerWarmImageError::StaleGeneration {
                expected: expected_generation,
                actual: self.generation.clone(),
            });
        }

        let next = match (self.state, operation) {
            (CloudWorkerWarmImageState::Missing, CloudWorkerWarmImageOperation::Allocate)
            | (CloudWorkerWarmImageState::CaptureUncertain, CloudWorkerWarmImageOperation::Recover) => {
                CloudWorkerWarmImageState::Pending
            }
            (CloudWorkerWarmImageState::Pending, CloudWorkerWarmImageOperation::Capture) => {
                CloudWorkerWarmImageState::Available
            }
            (CloudWorkerWarmImageState::Available, CloudWorkerWarmImageOperation::Fork) => {
                self.hold_count = self.hold_count.saturating_add(1);
                CloudWorkerWarmImageState::Available
            }
            (CloudWorkerWarmImageState::Available, CloudWorkerWarmImageOperation::Release) => {
                if self.hold_count == 0 {
                    return Err(CloudWorkerWarmImageError::NoActiveHold);
                }
                self.hold_count -= 1;
                CloudWorkerWarmImageState::Available
            }
            (CloudWorkerWarmImageState::Available, CloudWorkerWarmImageOperation::Retire) => {
                if self.hold_count != 0 {
                    return Err(CloudWorkerWarmImageError::HeldImage);
                }
                CloudWorkerWarmImageState::Retiring
            }
            (CloudWorkerWarmImageState::Retiring, CloudWorkerWarmImageOperation::Delete) => {
                CloudWorkerWarmImageState::Missing
            }
            (CloudWorkerWarmImageState::CaptureUncertain, CloudWorkerWarmImageOperation::Delete) => {
                self.cleanup_debt = true;
                CloudWorkerWarmImageState::Missing
            }
            _ => {
                return Err(CloudWorkerWarmImageError::InvalidOperation {
                    state: self.state,
                    operation,
                });
            }
        };

        self.state = next;
        self.generation.0 = self.generation.0.saturating_add(1);
        Ok(())
    }

    pub fn mark_capture_uncertain(
        &mut self,
        expected_generation: CloudWorkerWarmImageGeneration,
    ) -> Result<(), CloudWorkerWarmImageError> {
        if self.generation != expected_generation {
            return Err(CloudWorkerWarmImageError::StaleGeneration {
                expected: expected_generation,
                actual: self.generation.clone(),
            });
        }
        self.state = CloudWorkerWarmImageState::CaptureUncertain;
        self.cleanup_debt = true;
        self.generation.0 = self.generation.0.saturating_add(1);
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CloudWorkerWarmImageError {
    #[error("warm-image id must be non-empty")]
    InvalidImageId,
    #[error("warm-image generation is stale: expected {expected:?}, actual {actual:?}")]
    StaleGeneration {
        expected: CloudWorkerWarmImageGeneration,
        actual: CloudWorkerWarmImageGeneration,
    },
    #[error("warm-image is held by an active fork")]
    HeldImage,
    #[error("warm-image has no active fork hold")]
    NoActiveHold,
    #[error("operation {operation:?} is invalid for state {state:?}")]
    InvalidOperation {
        state: CloudWorkerWarmImageState,
        operation: CloudWorkerWarmImageOperation,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_and_apply_fences_stale_generations() {
        let mut image = CloudWorkerWarmImage::new("image-1").unwrap();
        image
            .compare_and_apply(
                CloudWorkerWarmImageGeneration(0),
                CloudWorkerWarmImageOperation::Allocate,
            )
            .unwrap();
        assert!(matches!(
            image.compare_and_apply(
                CloudWorkerWarmImageGeneration(0),
                CloudWorkerWarmImageOperation::Capture,
            ),
            Err(CloudWorkerWarmImageError::StaleGeneration { .. })
        ));
    }

    #[test]
    fn held_images_cannot_be_retired() {
        let mut image = CloudWorkerWarmImage::new("image-1").unwrap();
        image
            .compare_and_apply(
                CloudWorkerWarmImageGeneration(0),
                CloudWorkerWarmImageOperation::Allocate,
            )
            .unwrap();
        image
            .compare_and_apply(
                CloudWorkerWarmImageGeneration(1),
                CloudWorkerWarmImageOperation::Capture,
            )
            .unwrap();
        image
            .compare_and_apply(
                CloudWorkerWarmImageGeneration(2),
                CloudWorkerWarmImageOperation::Fork,
            )
            .unwrap();
        assert_eq!(
            image.compare_and_apply(
                CloudWorkerWarmImageGeneration(3),
                CloudWorkerWarmImageOperation::Retire,
            ),
            Err(CloudWorkerWarmImageError::HeldImage)
        );
    }

    #[test]
    fn uncertain_capture_is_recoverable_and_records_debt() {
        let mut image = CloudWorkerWarmImage::new("image-1").unwrap();
        image
            .mark_capture_uncertain(CloudWorkerWarmImageGeneration(0))
            .unwrap();
        assert_eq!(image.state, CloudWorkerWarmImageState::CaptureUncertain);
        assert!(image.cleanup_debt);
    }
}
