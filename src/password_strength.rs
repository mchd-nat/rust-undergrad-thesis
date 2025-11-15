// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct PasswordResult {
    pub password_input: bool,
    pub passed_checks: bool,
    pub error: bool,
}

pub async fn password_has_basic_checks(url: &str) -> PasswordResult {
    let client = Client::builder()
        .user_agent("DataSniffingCaramelo/1.0")
        .build()
        .unwrap();

    let html = match client.get(url).send().await {
        Ok(r) => match r.text().await {
            Ok(t) => t,
            Err(_) => return PasswordResult {
                password_input: false,
                passed_checks: false,
                error: true,
            }
        }
        Err(_) => return PasswordResult {
            password_input: false,
            passed_checks: false,
            error: true,
        }
    };

    let doc = Html::parse_document(&html);

    let input_sel = Selector::parse("input[type=password]").unwrap();
    let has_input = doc.select(&input_sel).next().is_some();

    if !has_input {
        return PasswordResult {
            password_input: false,
            passed_checks: false,
            error: false,
        };
    }

    let text = doc.root_element().text().collect::<Vec<_>>().join(" ").to_lowercase();

    let strict_rules = [
        "mínimo de",
        "caracteres",
        "número",
        "especial",
        "letra maiúscula",
        "complexidade",
        "requisitos de senha",
        "password must",
        "at least",
        "uppercase",
        "lowercase",
        "digit",
        "special character",
    ];

    let mut passed = false;
    for rule in strict_rules {
        if text.contains(rule) {
            passed = true;
            break;
        }
    }

    PasswordResult {
        password_input: true,
        passed_checks: passed,
        error: false,
    }
}
