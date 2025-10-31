use crate::URL;

use scraper::{Html, Selector};
use serde_json::json;
use serde::Serialize;
use std::collections::VecDeque;
use std::collections::{HashSet};
use reqwest::Url;

#[derive(Serialize)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub fn parse(body: String) {
    let document = Html::parse_document(&body);
    let full_text: String = document.root_element().text().collect::<Vec<_>>().join(" ").trim().to_string().to_lowercase();

    checklist(document, full_text);
}

pub fn checklist(document:Html, full_text: String) {
    let mut results = vec![];
    let mut has_privacy_policy = false;
    let mut has_cookie_refusal = false;

    let anchors = Selector::parse("a").unwrap();
    let mut to_visit: VecDeque<&str> = VecDeque::new();
    let mut visited = HashSet::new();
    to_visit.insert(0, URL);

    while let Some(base_url) = to_visit.pop_front() {
        for link in document.select(&anchors) {
            let abs_link = normalize_url(&link, &base_url);
            if !visited.contains(&abs_link) {
                println!("Visitando {}", &abs_link);

                if !has_privacy_policy {
                    has_privacy_policy = full_text.contains("política de privacidade")
                    || full_text.contains("notificação de privacidade");
                }
                if !has_cookie_refusal {
                    has_cookie_refusal = full_text.contains("cookies") 
                    && (
                        full_text.contains("recusar")
                        || full_text.contains("negar")
                        || full_text.contains("não aceitar")
                    );
                }

                visited.insert(abs_link);
            }
        }
    }

    results.push(CheckResult { check: "Política de Privacidade".into(), passed: has_privacy_policy, error: None });
    results.push(CheckResult { check: "Opção de recusar coleta de Cookies".into(), passed: has_cookie_refusal, error: None });

    let results_json = json!({ "ready": true, "results": results });
    println!("{}", results_json);
}

fn normalize_url(link: &scraper::ElementRef, base_url: &str) -> String {
    if let Some(href) = link.value().attr("href") {
        if let Ok(url) = Url::parse(href) {
            url.to_string()
        } else if let Ok(base) = Url::parse(base_url) {
            if let Ok(joined) = base.join(href) {
                joined.to_string()
            } else {
                href.to_string()
            }
        } else {
            href.to_string()
        }
    } else {
        String::new()
    }
}