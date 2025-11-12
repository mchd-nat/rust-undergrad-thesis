// SPDX-FileCopyrightText: 2025 Natália Silva Machado
// SPDX-License-Identifier: GPL-3.0-or-later

use scraper::{Html, Selector};

pub struct PasswordResult {
    pub password_input: bool,
    pub passed_checks: bool,
    pub error: bool
}

pub async fn password_has_basic_checks(url: &str) -> PasswordResult {
    let client = reqwest::Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap();

    let resp = match client.get(url)
        .header("X-Crawler", "DataSniffingCaramelo (smachadonatalia@gmail.com)")
        .send()
        .await {
            Ok(r) => r,
            Err(_) => return {
                PasswordResult {
                    password_input: false,
                    passed_checks: false,
                    error: true
                }
            }
    };

    let body = match resp.text().await {
        Ok(r) => r,
        Err(_) => return {
            PasswordResult {
                password_input: false,
                passed_checks: false,
                error: true
            }
        }
    };

    let document = Html::parse_document(&body);
    let input_sel = match Selector::parse(r#"input[type="password"]"#) {
        Ok(sel) => sel,
        Err(_) => return {
            PasswordResult {
                password_input: false,
                passed_checks: false,
                error: true
            }
        }
    };

    let some_password_input = match document.select(&input_sel).next() {
        Some(r) => r,
        None => {
            return PasswordResult {
                password_input: false,
                passed_checks: false,
                error: false,
            }
        }
    };

    let pattern = some_password_input.value().attr("pattern");
    let minlength = some_password_input
        .value()
        .attr("minlength")
        .and_then(|s| s.parse::<usize>().ok());

    let script_sel = match Selector::parse("script") {
        Ok(sel) => sel,
        Err(_) => return {
            PasswordResult {
                password_input: false,
                passed_checks: false,
                error: true
            }
        }
    };

    let mut js_has_strength = false;
    for script in document.select(&script_sel) {
        if let Some(src) = script.value().attr("src") {
            if src.to_lowercase().contains("zxcvbn") {
                js_has_strength = true;
                break;
            }
        }
        let inline = script.text().collect::<Vec<_>>().join(" ");
        let inline_lower = inline.to_lowercase();
        if inline_lower.contains("password") && inline.to_lowercase().contains("strength") {
            js_has_strength = true;
            break;
        }
    }

    PasswordResult {
        password_input: true,
        passed_checks: pattern.is_some() || minlength.is_some() || js_has_strength,
        error: false
    }
}
