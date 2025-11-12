// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::password_strength::password_has_basic_checks;
use crate::password_strength::ChecksResult;

use reqwest::Url;
use reqwest::Response as R;
use robotstxt_rs::RobotsTxt;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub async fn check_cookie_consent(response: &R) -> bool {
    let r = response;
    let cookie_headers = r.headers().get_all(reqwest::header::SET_COOKIE);
    
    cookie_headers.iter().next().is_none()
}

pub async fn check_robots(url: String) -> bool {
    let robots_url = url.as_str().to_owned() + "/robots.txt";
    let robots = RobotsTxt::from_url(&robots_url).await;

    return robots.expect("Erro ao checar robots.txt").can_fetch("DataSniffingCaramelo", url.as_str());
}

pub async fn run_crawler(url: &str) -> Vec<CheckResult> {
    let mut results = vec![];
    
    let base_url = match Url::parse(&url) {
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

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("DataSniffingCaramelo (natalias2@mx2.unisc.br)")
        .build()
        .unwrap();
    
    if check_robots(base_url.to_string()).await {
        let mut has_privacy_policy = false;
        // let mut has_cookie_refusal = false;
        let mut has_password_policy: ChecksResult = ChecksResult{
            password_input: true,
            passed_checks: false,
            error: false
        };

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
                            || full_text.contains("notificação de privacidade")
                            || full_text.contains("privacy policy");
                            
                        /* if full_text.contains("cookies") {
                            if full_text.contains("recusar")
                            || full_text.contains("negar")
                            || full_text.contains("não aceitar")
                            || full_text.contains("rejeitar")
                            || full_text.contains("refuse")
                            || full_text.contains("reject") {
                                has_cookie_refusal = true;
                            }
                        } */
                    }
                    Err(e) => {
                        eprintln!("Erro ao ler página {}: {}", base_url, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Erro ao tentar alcançar {}: {}", base_url, e);
            }
        }

        let base_domain = base_url.domain().unwrap_or("").to_string();
        
        let mut to_visit: VecDeque<String> = VecDeque::new();
        let mut visited: HashSet<String> = HashSet::new();
        to_visit.push_back(url.to_string());

        let max_pages = 30;
        let mut pages_visited = 0;

        let mut respects_cookie_consent = false;

        while let Some(current_url) = to_visit.pop_front() {
            if pages_visited >= max_pages {
                break;
            }

            if visited.contains(&current_url) {
                continue;
            }

            visited.insert(current_url.clone());
            pages_visited += 1;

            println!("Visitando: {}", current_url);
            if check_robots(current_url.to_string()).await {
                match client.get(&current_url).send().await {
                    Ok(response) => {
                        //if has_cookie_refusal {
                            respects_cookie_consent = check_cookie_consent(&response).await;
                        //}

                        let formatted_string = &current_url
                            .to_lowercase()
                            .replace(' ', "");

                        if formatted_string.contains("cadastro") 
                            || formatted_string.contains("signup")
                            || formatted_string.contains("criar")
                            || formatted_string.contains("nova") {
                                has_password_policy = password_has_basic_checks(&current_url).await;
                            }

                        match response.text().await {
                            Ok(html_content) => {
                                let document = Html::parse_document(&html_content);

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
                                eprintln!("Erro ao ler página {}: {}", current_url, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Erro ao tentar alcançar {}: {}", current_url, e);
                    }
                }
            } 
        }

        results.push(CheckResult {
            check: "Política de Privacidade".into(),
            passed: has_privacy_policy,
            error: None,
        });
        
        /* if !respects_cookie_consent {
            results.push(CheckResult {
                check: "Opção de recusar coleta de Cookies".into(),
                passed: has_cookie_refusal,
                error: None,
            });
        } */

        results.push(CheckResult {
            check: "Coleta cookies somente após consentimento do usuário".into(),
            passed: respects_cookie_consent,
            error: None,
        });

        if has_password_policy.password_input && !has_password_policy.error {
            results.push(CheckResult {
                check: "Tem uma política de força de senha".into(),
                passed: has_password_policy.passed_checks,
                error: None,
            });
        }
    } else {
        results.push(CheckResult {
            check: "Website inserido não permite a ação de webcrawlers".into(),
            passed: false,
            error: None,
        });
    }

    results
}