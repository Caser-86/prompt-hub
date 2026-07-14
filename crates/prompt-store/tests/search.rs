use prompt_domain::{
    Actor, AuditAction, Compatibility, CompatibilityStatus, EffectivenessStatus, Prompt,
    PromptContent, PromptSource, SourceKind, ValidationRecord,
};
use prompt_store::{Database, SearchFilters, SearchQuery};
use serde::Deserialize;
use tempfile::tempdir;
use time::macros::datetime;

#[derive(Debug, Deserialize)]
struct Fixture {
    title: String,
    body: String,
    category: String,
    tags: Vec<String>,
    tool: String,
    model: String,
    effectiveness: EffectivenessStatus,
    rating: Option<u8>,
}

fn fixtures() -> Vec<Fixture> {
    serde_json::from_str(include_str!("../../../tests/fixtures/search-zh.json")).unwrap()
}

fn seed() -> prompt_store::PromptRepository {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    for (index, fixture) in fixtures().into_iter().enumerate() {
        let at = datetime!(2026-07-15 00:00 UTC) + time::Duration::minutes(index as i64);
        let mut prompt = Prompt::new_inbox(
            PromptContent::new(
                fixture.title,
                fixture.body,
                Some("中文检索测试数据".to_owned()),
                Some(fixture.category),
                fixture.tags,
            )
            .unwrap(),
            PromptSource::new(SourceKind::Manual, "测试资料", None, at).unwrap(),
            Actor::User,
            at,
        );
        prompt
            .add_compatibility(
                Compatibility::new(
                    fixture.tool,
                    Some(fixture.model),
                    CompatibilityStatus::Confirmed,
                    None,
                    Some(at),
                )
                .unwrap(),
                Actor::User,
                at,
            )
            .unwrap();
        prompt
            .record_validation(
                ValidationRecord::new(fixture.effectiveness, fixture.rating, None, at).unwrap(),
                Actor::User,
                at,
            )
            .unwrap();
        prompt.publish(fixture.effectiveness, at).unwrap();
        repository.save(&prompt, AuditAction::Created).unwrap();
    }
    repository
}

#[test]
fn finds_chinese_substrings_with_a_highlighted_snippet() {
    let repository = seed();

    let page = repository
        .search(SearchQuery::new("潜在缺陷"))
        .expect("Chinese FTS search should succeed");

    assert_eq!(page.total, 2);
    assert_eq!(page.hits[0].title, "代码审查专家");
    assert!(page.hits[0].snippet.contains("<mark>潜在缺陷</mark>"));
}

#[test]
fn uses_a_safe_fallback_for_two_character_queries() {
    let repository = seed();

    let page = repository.search(SearchQuery::new("审查")).unwrap();

    assert_eq!(page.total, 2);
    assert!(page.hits.iter().all(|hit| hit.title.contains("审查")));
}

#[test]
fn combines_tool_model_effectiveness_rating_and_source_filters() {
    let repository = seed();
    let query = SearchQuery::new("代码").with_filters(SearchFilters {
        tool: Some("Codex".to_owned()),
        model: Some("gpt-5.6-sol".to_owned()),
        effectiveness: Some(EffectivenessStatus::Effective),
        minimum_rating: Some(4),
        source_kind: Some(SourceKind::Manual),
        ..SearchFilters::default()
    });

    let page = repository.search(query).unwrap();

    assert_eq!(page.total, 1);
    assert_eq!(page.hits[0].title, "代码审查专家");
}

#[test]
fn ranks_effective_prompts_before_ineffective_prompts_for_equal_text_matches() {
    let repository = seed();

    let page = repository.search(SearchQuery::new("检查当前代码")).unwrap();

    assert_eq!(page.hits[0].effectiveness, EffectivenessStatus::Effective);
    assert_eq!(page.hits[1].effectiveness, EffectivenessStatus::Ineffective);
}

#[test]
fn pagination_is_deterministic_and_reports_the_full_total() {
    let repository = seed();

    let first = repository
        .search(SearchQuery::new("").with_page(1, 0))
        .unwrap();
    let second = repository
        .search(SearchQuery::new("").with_page(1, 1))
        .unwrap();
    let repeat = repository
        .search(SearchQuery::new("").with_page(1, 0))
        .unwrap();

    assert_eq!(first.total, 3);
    assert_ne!(first.hits[0].id, second.hits[0].id);
    assert_eq!(first.hits[0].id, repeat.hits[0].id);
}

#[test]
fn detects_and_rebuilds_an_inconsistent_search_index() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("search-rebuild.db");
    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    let at = datetime!(2026-07-15 00:00 UTC);
    let prompt = Prompt::new_inbox(
        PromptContent::new(
            "索引恢复测试",
            "能够重新找到这条提示词",
            None,
            Some("测试".to_owned()),
            vec!["恢复".to_owned()],
        )
        .unwrap(),
        PromptSource::new(SourceKind::Manual, "测试资料", None, at).unwrap(),
        Actor::User,
        at,
    );
    repository.save(&prompt, AuditAction::Created).unwrap();
    drop(repository);

    let raw = rusqlite::Connection::open(&path).unwrap();
    raw.execute("DELETE FROM prompt_fts", []).unwrap();
    drop(raw);

    let database = Database::open(&path).unwrap();
    let mut repository = database.into_repository();
    assert!(!repository.search_index_is_consistent().unwrap());

    repository.rebuild_search_index().unwrap();

    assert!(repository.search_index_is_consistent().unwrap());
    assert_eq!(
        repository
            .search(SearchQuery::new("重新找到"))
            .unwrap()
            .total,
        1
    );
}
