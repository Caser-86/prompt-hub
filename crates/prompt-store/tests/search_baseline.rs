use std::time::Instant;

use prompt_domain::{Actor, AuditAction, Prompt, PromptContent, PromptSource, SourceKind};
use prompt_store::{Database, SearchQuery};
use time::macros::datetime;

#[test]
#[ignore = "explicit local performance baseline"]
fn reports_a_repeatable_search_baseline_for_one_thousand_chinese_prompts() {
    let database = Database::open_in_memory().unwrap();
    let mut repository = database.into_repository();
    let at = datetime!(2026-07-15 00:00 UTC);

    for index in 0..1_000 {
        let prompt = Prompt::new_inbox(
            PromptContent::new(
                format!("性能检索样本 {index}"),
                format!("第 {index} 条中文提示词用于性能检索和索引验证。"),
                None,
                Some("基准".to_owned()),
                vec!["性能".to_owned(), "检索".to_owned()],
            )
            .unwrap(),
            PromptSource::new(SourceKind::Manual, "基准数据", None, at).unwrap(),
            Actor::User,
            at,
        );
        repository.save(&prompt, AuditAction::Created).unwrap();
    }

    let started = Instant::now();
    let page = repository
        .search(SearchQuery::new("性能检索").with_page(20, 0))
        .unwrap();
    let elapsed = started.elapsed();

    assert_eq!(page.total, 1_000);
    assert_eq!(page.hits.len(), 20);
    println!("SEARCH_BASELINE_1000_US={}", elapsed.as_micros());
}
