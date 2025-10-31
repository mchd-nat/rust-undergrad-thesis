mod utils;

fn main() {
    let url = "https://www.amazon.com.br/";
    match reqwest::blocking::get(url) {
        Ok(r) => {
            let bytes = r.bytes().unwrap();
            let html_content = String::from_utf8(bytes.to_vec());

            match html_content {
                Ok(c) => {
                    utils::parse(c);
                },
                Err(e) => {
                    eprint!("Decodificação para UTF=8 falhou: {}", e);
                }
            }
        },
        Err(e) => {
            eprint!("Solicitação para {} falhou: {}", url, e);
        } 
    };
}


