use prompt_ai::{AiDraft, DraftDestination};
use prompt_ai::{
    AiProvider, CredentialStore, DraftGenerator, GenerationOutput, GenerationRequest, ProviderError,
};
use secrecy::SecretString;
use time::macros::datetime;

#[test]
fn ai_results_are_always_inbox_drafts() {
    let draft = AiDraft::new(
        "代码审查",
        "审查当前变更",
        "gpt-5",
        "根据用户的代码审查需求生成",
        datetime!(2026-07-15 00:00 UTC),
    )
    .unwrap();

    assert_eq!(draft.destination(), DraftDestination::Inbox);
}

struct Provider;
#[async_trait::async_trait]
impl AiProvider for Provider {
    async fn generate(
        &self,
        _: GenerationRequest,
        _: SecretString,
        _: tokio::sync::watch::Receiver<bool>,
    ) -> Result<GenerationOutput, ProviderError> {
        Ok(GenerationOutput {
            title: "AI 草稿".to_owned(),
            body: "待审核内容".to_owned(),
        })
    }
}
struct Credentials;
impl CredentialStore for Credentials {
    fn save(&self, _: &str, _: SecretString) -> Result<(), prompt_ai::CredentialError> {
        Ok(())
    }
    fn load(&self, _: &str) -> Result<Option<SecretString>, prompt_ai::CredentialError> {
        Ok(Some(SecretString::from("secret")))
    }
    fn remove(&self, _: &str) -> Result<(), prompt_ai::CredentialError> {
        Ok(())
    }
}

#[tokio::test]
async fn provider_generation_cannot_return_any_destination_except_inbox() {
    let generator = DraftGenerator::new(Provider, Credentials);
    let (_, cancellation) = tokio::sync::watch::channel(false);
    let draft = generator
        .generate_cancellable(
            "openai",
            GenerationRequest {
                instruction: "优化提示词".to_owned(),
                input_summary: "不含正文的摘要".to_owned(),
                model: "gpt-5".to_owned(),
            },
            datetime!(2026-07-15 00:00 UTC),
            cancellation,
        )
        .await
        .unwrap();
    assert_eq!(draft.destination(), DraftDestination::Inbox);
}

#[tokio::test]
async fn cancelled_generation_does_not_create_a_draft() {
    let generator = DraftGenerator::new(Provider, Credentials);
    let (cancellation_sender, cancellation) = tokio::sync::watch::channel(true);

    let error = generator
        .generate_cancellable(
            "openai",
            GenerationRequest {
                instruction: "优化提示词".to_owned(),
                input_summary: "不含正文的摘要".to_owned(),
                model: "gpt-5".to_owned(),
            },
            datetime!(2026-07-15 00:00 UTC),
            cancellation,
        )
        .await
        .unwrap_err();

    drop(cancellation_sender);
    assert!(matches!(error, prompt_ai::GenerationError::Cancelled));
}
