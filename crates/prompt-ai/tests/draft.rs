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
impl AiProvider for Provider {
    fn generate(
        &self,
        _: GenerationRequest,
        _: SecretString,
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

#[test]
fn provider_generation_cannot_return_any_destination_except_inbox() {
    let generator = DraftGenerator::new(Provider, Credentials);
    let draft = generator
        .generate(
            "openai",
            GenerationRequest {
                instruction: "优化提示词".to_owned(),
                input_summary: "不含正文的摘要".to_owned(),
                model: "gpt-5".to_owned(),
            },
            datetime!(2026-07-15 00:00 UTC),
        )
        .unwrap();
    assert_eq!(draft.destination(), DraftDestination::Inbox);
}
