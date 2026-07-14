use prompt_hub_desktop_lib::commands::get_application_status;
use prompt_store::LATEST_SCHEMA_VERSION;
use serde_json::json;

#[test]
fn application_status_uses_the_shared_frontend_wire_contract() {
    let status = get_application_status();

    assert_eq!(
        serde_json::to_value(status).unwrap(),
        json!({
            "appVersion": env!("CARGO_PKG_VERSION"),
            "databaseSchemaVersion": LATEST_SCHEMA_VERSION,
            "offlineCapable": true,
        })
    );
}
