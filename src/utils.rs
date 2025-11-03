use scraper::{Html};
use serde::Serialize;
use reqwest::Url;

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub async fn run_crawler(url: &str) -> Vec<CheckResult> {
    let mut results = vec![];
    
    let base_url = match Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            results.push(CheckResult {
                check: "Erro ao processar URL".into(),
                passed: false,
                error: Some(e.to_string()),
            });
            return results;
        }
    };

    let mut has_privacy_policy = false;
    let mut has_cookie_refusal = false;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    match client.get(base_url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(html_content) => {
                    let document = Html::parse_document(&html_content);
                    let full_text: String = document
                        .root_element()
                        .text()
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string()
                        .to_lowercase();

                    if !has_privacy_policy {
                        has_privacy_policy = full_text.contains("política de privacidade")
                            || full_text.contains("notificação de privacidade")
                            || full_text.contains("privacy policy");
                    }
                    if !has_cookie_refusal {
                        if full_text.contains("cookies") {
                            if full_text.contains("recusar")
                            || full_text.contains("negar")
                            || full_text.contains("não aceitar")
                            || full_text.contains("rejeitar") {
                                has_cookie_refusal = true;
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erro lendo página: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Erro buscando: {}", e);
        }
    }

    results.push(CheckResult {
        check: "Política de Privacidade".into(),
        passed: has_privacy_policy,
        error: None,
    });
    
    results.push(CheckResult {
        check: "Opção de recusar coleta de Cookies".into(),
        passed: has_cookie_refusal,
        error: None,
    });

    results
}