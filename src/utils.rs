// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

use fantoccini::{Client, ClientBuilder, Locator};
use robotstxt_rs::RobotsTxt;
use serde::Serialize;
use std::{collections::{HashSet, VecDeque}};
use tokio::time::{Duration, sleep};
use url::Url;

use crate::password_strength::{PasswordResult, password_has_basic_checks};

#[derive(Serialize, Clone)]
pub struct CheckResult {
    pub check: String,
    pub passed: bool,
    pub error: Option<String>,
}

pub async fn check_robots(url: String) -> bool {
    let robots_url = format!("{}/robots.txt", url.trim_end_matches('/'));
    match RobotsTxt::from_url(&robots_url).await {
        Ok(r) => r.can_fetch("DataSniffingCaramelo", &url),
        Err(_) => false,
    }
}

pub fn is_potential_signup(url: &String) -> bool {
    let formatted: String = url
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    formatted.contains("cadastro")
        || formatted.contains("signup")
        || formatted.contains("criarconta")
        || formatted.contains("novaconta")
        || formatted.contains("cadastrarse")
}

pub async fn get_page_text(client: &Client) -> String {
    match client.find(Locator::Css("body")).await {
        Ok(body) => body.text().await.unwrap_or_default(),
        Err(_) => String::new(),
    }
}

pub async fn allows_cookie_refusal(client: &Client) -> bool {
    sleep(Duration::from_millis(1500)).await;

    let selectors = [
        "#onetrust-banner-sdk",
        "#CybotCookiebotDialog",
        ".didomi-popup",
        ".qc-cmp2-container",
        ".trustarc-banner",
        ".cookie-consent",
        ".cookie-banner",
        ".cookie-notice",
    ];

    let refusal = [
        "recusar",
        "negar",
        "não aceitar",
        "rejeitar",
        "refuse",
        "reject",
    ];

    for sel in selectors {
        if let Ok(banner) = client.find(Locator::Css(sel)).await {
            if let Ok(buttons) = banner.find_all(Locator::Css("button")).await {
                for b in buttons {
                    if let Ok(text) = b.text().await {
                        let t = text.to_lowercase();
                        if refusal.iter().any(|k| t.contains(k)) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    if let Ok(divs) = client.find_all(Locator::Css("button")).await {
        for el in divs {
            if let Ok(t) = el.text().await {
                let tl = t.to_lowercase();
                if refusal.iter().any(|k| tl.contains(k)) {
                    return true;
                }
            }
        }
    }

    false
}

pub async fn check_cookie_consent(client: &Client) -> bool {

    match client.get_all_cookies().await {
        Ok(c) => c.is_empty(),
        Err(_) => false,
    }
}

pub async fn run_crawler(url: &str) -> Vec<CheckResult> {
    let mut results = vec![];

    let client = match ClientBuilder::native()
        .connect("http://localhost:9515")
        .await {
            Ok(c) => c,
            Err(e) => {
                results.push(CheckResult {
                    check: "Erro ao iniciar navegador".into(),
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

        let _ = client.close().await;
        return results;
    }

    if let Err(e) = client.goto(url).await {
        results.push(CheckResult { 
            check: "Erro ao abrir página inicial".into(),
            passed: false,
            error: Some(e.to_string()),
        });
        let _ = client.close().await;
        return results;
    }

    sleep(Duration::from_millis(800)).await;
    let text = get_page_text(&client).await.to_lowercase();

    let has_privacy_policy = text.contains("política de privacidade")
        || text.contains("notificação de privacidade")
        || text.contains("privacy policy");

    let has_cookie_refusal = allows_cookie_refusal(&client).await;
    let respects_cookie_consent = check_cookie_consent(&client).await;
    let mut password_result = PasswordResult {
        password_input: false,
        passed_checks: false,
        error: false,
    };

    if is_potential_signup(&url.to_string()) {
        password_result = password_has_basic_checks(url).await;
    }

    let base_domain = Url::parse(url)
        .ok()
        .and_then(|u| u.domain().map(|s| s.to_string()))
        .unwrap_or_default();

    let mut visited = HashSet::<String>::new();
    let mut queue = VecDeque::<String>::new();
    queue.push_back(url.to_string());

    let max_pages = 20;
    let mut count = 0;

    while let Some(current) = queue.pop_front() {
        if visited.contains(&current) || count >= max_pages {
            continue;
        }

        visited.insert(current.clone());
        count += 1;

        if client.goto(&current).await.is_err() {
            continue;
        }

        sleep(Duration::from_millis(500)).await;

        if let Ok(links) = client.find_all(Locator::Css("a")).await {
            for a in links {
                if let Ok(href) = a.attr("href").await {
                    if let Some(h) = href {
                        if let Ok(abs) = Url::parse(&current)
                            .and_then(|b| b.join(&h))
                        {
                            let s = abs.to_string();
                            if s.contains(&base_domain) && !visited.contains(&s) {
                                queue.push_back(s);
                            }
                        }
                    }
                }
            }
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

    if password_result.password_input && !password_result.error {
        results.push(CheckResult {
            check: "Tem uma política de força de senha".into(),
            passed: password_result.passed_checks,
            error: None,
        });
    }

    let _ = client.close().await;
    results
}
