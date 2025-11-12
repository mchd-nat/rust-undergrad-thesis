// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::password_strength::password_has_basic_checks;
use crate::password_strength::PasswordResult;

use chromiumoxide::{Browser, BrowserConfig, Page};
use chromiumoxide::cdp::browser_protocol::network::Cookie;
use chromiumoxide::handler::viewport::Viewport;
use futures_util::StreamExt;
use robotstxt_rs::RobotsTxt;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};
use url::Url;

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub async fn check_cookie_consent(page: &Page) -> bool {
    let cookies_before: Vec<Cookie> = page.get_cookies().await.expect("Erro ao buscar cookies");
    
    cookies_before.is_empty()
}

pub async fn check_robots(url: String) -> bool {
    let robots_url = format!("{}/robots.txt", url.trim_end_matches('/'));
    match RobotsTxt::from_url(&robots_url).await {
        Ok(r) => r.can_fetch("DataSniffingCaramelo", &url),
        Err(e) => {
            eprintln!("Erro ao checar robots.txt: {}", e);
            false
        }
    }
}

pub async fn check_url(url: &String) -> bool {
    let formatted_url: String = url
                                .to_lowercase()
                                .chars()
                                .filter(|c| c.is_alphanumeric())
                                .collect();

    formatted_url.contains("cadastro")
        || formatted_url.contains("signup")
        || formatted_url.contains("criarconta")
        || formatted_url.contains("novaconta")
}

pub async fn allows_cookie_refusal(text: &String) -> bool {
    text.contains("recusar")
        || text.contains("negar")
        || text.contains("não aceitar")
        || text.contains("rejeitar")
        || text.contains("refuse")
        || text.contains("reject")
}

pub async fn run_crawler(url: &str) -> Vec<CheckResult> {
    let mut results = vec![];

    let config = BrowserConfig::builder()
    .viewport(Viewport {
        width: 1280,
        height: 720,
        device_scale_factor: Some(1.0),
        emulating_mobile: false,
        has_touch: false,
        is_landscape: true,
    })
    .no_sandbox()
    .build()
    .unwrap();
    
    let (browser, mut handler) = match Browser::launch(config).await {
        Ok(b) => b,
        Err(e) => {
            results.push(CheckResult {
                check: "Erro ao iniciar navegador".into(),
                passed: false,
                error: Some(e.to_string()),
            });
            return results;
        }
    };

    tokio::spawn(async move {
        while let Some(_event) = handler.next().await {}
    });

    let page = match browser.new_page(url).await {
        Ok(p) => p,
        Err(e) => {
            results.push(CheckResult {
                check: "Erro ao abrir página inicial".into(),
                passed: false,
                error: Some(e.to_string()),
            });
            return results;
        }
    };

    if !check_robots(url.to_string()).await {
        results.push(CheckResult {
            check: "Website inserido não permite a ação de webcrawlers".into(),
            passed: false,
            error: None,
        });
        return results;
    }

    let html_content = match page.content().await {
        Ok(c) => c,
        Err(e) => {
            results.push(CheckResult {
                check: "Erro ao obter conteúdo da página".into(),
                passed: false,
                error: Some(e.to_string()),
            });
            return results;
        }
    };

    let document = Html::parse_document(&html_content);
    let full_text: String = document.root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    let has_privacy_policy = full_text.contains("política de privacidade")
        || full_text.contains("notificação de privacidade")
        || full_text.contains("privacy policy");

    let has_cookie_refusal = allows_cookie_refusal(&full_text).await;
    let respects_cookie_consent = check_cookie_consent(&page).await;

    let mut has_password_policy = PasswordResult {
        password_input: true,
        passed_checks: false,
        error: false,
    };

    let base_url = url.to_string();
    let base_domain = url.split("/").nth(2).unwrap_or("").to_string();

    if check_url(&base_url).await {
        has_password_policy = password_has_basic_checks(&base_url).await;
    }

    let mut to_visit: VecDeque<String> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();
    to_visit.push_back(url.to_string());

    let max_pages = 30;
    let mut pages_visited = 0;

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

        match browser.new_page(&current_url).await {
            Ok(p) => {
                if check_url(&current_url).await {
                    has_password_policy = password_has_basic_checks(&base_url).await;
                }

                if let Ok(html) = p.content().await {
                    let doc = Html::parse_document(&html);
                    let anchors = Selector::parse("a").unwrap();
                    let base = Url::parse(&current_url).unwrap();

                    for link in doc.select(&anchors) {
                        if let Some(href) = link.value().attr("href") {
                            
                            if let Ok(resolved) = base.join(href) {
                                let resolved_str = resolved.as_str().to_string();

                                if resolved_str.contains(&base_domain)
                                    && !visited.contains(&resolved_str)
                                {
                                    to_visit.push_back(resolved_str);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Erro ao visitar {}: {}", current_url, e),
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

    results
}