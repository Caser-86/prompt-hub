use serde::Serialize;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftDestination {
    Inbox,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiDraft {
    title: String,
    body: String,
    model: String,
    generated_at: OffsetDateTime,
    input_summary: String,
    destination: DraftDestination,
}

impl AiDraft {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        model: impl Into<String>,
        input_summary: impl Into<String>,
        generated_at: OffsetDateTime,
    ) -> Result<Self, DraftError> {
        let title = required(title.into(), DraftError::TitleRequired)?;
        let body = required(body.into(), DraftError::BodyRequired)?;
        let model = required(model.into(), DraftError::ModelRequired)?;
        let input_summary = required(input_summary.into(), DraftError::InputSummaryRequired)?;
        Ok(Self {
            title,
            body,
            model,
            generated_at,
            input_summary,
            destination: DraftDestination::Inbox,
        })
    }

    #[must_use]
    pub const fn destination(&self) -> DraftDestination {
        self.destination
    }
}

fn required(value: String, error: DraftError) -> Result<String, DraftError> {
    let value = value.trim().to_owned();
    if value.is_empty() { Err(error) } else { Ok(value) }
}

#[derive(Debug, Error)]
pub enum DraftError {
    #[error("AI draft title is required")]
    TitleRequired,
    #[error("AI draft body is required")]
    BodyRequired,
    #[error("AI model is required")]
    ModelRequired,
    #[error("AI input summary is required")]
    InputSummaryRequired,
}
