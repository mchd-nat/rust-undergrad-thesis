//use scraper::{Html, Selector};
use scraper::Html;
use serde::Serialize;
//use std::collections::{HashSet, VecDeque};
use std::collections::VecDeque;
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

    match client.get(base_url.clone()).send().await {
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

                    has_privacy_policy = full_text.contains("política de privacidade")
                        || full_text.contains("notificação de privacidade");
                        
                    if full_text.contains("cookies") {
                        if full_text.contains("recusar")
                        || full_text.contains("negar")
                        || full_text.contains("não aceitar")
                        || full_text.contains("rejeitar") {
                            has_cookie_refusal = true;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading page {}: {}", base_url, e);
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching {}: {}", base_url, e);
        }
    }

    //let base_domain = base_url.domain().unwrap_or("").to_string();
    
    let mut to_visit: VecDeque<String> = VecDeque::new();
    //let mut visited: HashSet<String> = HashSet::new();
    to_visit.push_back(url.to_string());

    //let max_pages = 30;
    //let mut pages_visited = 0;
    let pages_visited = 0;

    /* while let Some(current_url) = to_visit.pop_front() {
        if pages_visited >= max_pages {
            break;
        }

        if visited.contains(&current_url) {
            continue;
        }

        visited.insert(current_url.clone());
        pages_visited += 1;

        println!("Visiting: {}", current_url);

        match client.get(&current_url).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(html_content) => {
                        let document = Html::parse_document(&html_content);
                        /* let full_text: String = document
                            .root_element()
                            .text()
                            .collect::<Vec<_>>()
                            .join(" ")
                            .trim()
                            .to_string()
                            .to_lowercase(); */

                        // add here

                        let anchors = Selector::parse("a").unwrap();
                        for link in document.select(&anchors) {
                            if let Some(href) = link.value().attr("href") {
                                if let Ok(absolute_url) = base_url.join(href) {
                                    let link_domain = absolute_url.domain().unwrap_or("");
                                    
                                    if link_domain == base_domain && !visited.contains(absolute_url.as_str()) {
                                        to_visit.push_back(absolute_url.to_string());
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading page {}: {}", current_url, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", current_url, e);
            }
        }
    } */

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

    println!("Crawler finished. Visited {} pages.", pages_visited);
    results
}