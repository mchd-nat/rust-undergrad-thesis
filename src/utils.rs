use scraper::{Html};
use serde_json::json;
use serde::Serialize;

#[derive(Serialize)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub fn parse(body: String) {
    let document = Html::parse_document(&body);
    let full_text: String = document.root_element().text().collect::<Vec<_>>().join(" ").trim().to_string().to_lowercase();

    checklist(full_text);
}

pub fn checklist(full_text: String) {
    let mut results = vec![];

    let has_privacy_policy = full_text.contains("política de privacidade")
    || full_text.contains("notificação de privacidade");
    let has_cookie_refusal = full_text.contains("cookies") 
    && (
        full_text.contains("recusar")
        || full_text.contains("negar")
        || full_text.contains("não aceitar")
    );

    results.push(CheckResult { check: "Política de Privacidade".into(), passed: has_privacy_policy, error: None });
    results.push(CheckResult { check: "Opção de recusar coleta de Cookies".into(), passed: has_cookie_refusal, error: None });

    let results_json = json!({ "ready": true, "results": results });
    println!("{}", results_json);
}