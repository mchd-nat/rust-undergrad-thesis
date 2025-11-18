# Data-sniffing Caramelo 🐕

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) ![HTML5](https://img.shields.io/badge/html5-%23E34F26.svg?style=for-the-badge&logo=html5&logoColor=white) ![CSS3](https://img.shields.io/badge/css3-%231572B6.svg?style=for-the-badge&logo=css3&logoColor=white) ![JavaScript](https://img.shields.io/badge/javascript-%23323330.svg?style=for-the-badge&logo=javascript&logoColor=%23F7DF1E) ![Render](https://img.shields.io/badge/Render-%46E3B7.svg?style=for-the-badge&logo=render&logoColor=white)

## 🛠 Sobre a ferramenta

*Data-Sniffing Caramelo* é seu cão farejador virtual que inspeciona sites da web para encontrar infrações à [Lei Geral de Proteção de Dados](https://www.gov.br/esporte/pt-br/acesso-a-informacao/lgpd). 

A ferramenta foi criada como parte do meu TCC de Ciência da Computação. Atualmente, ela é capaz de fazer quatro checagens:
          
- [x] 📑 Presença (ou ausência) de uma política de privacidade [^1]
- [x] 🍪 Presença (ou ausência) de aviso sobre a coleta de cookies e a opção de recusar tal coleta [^2]
- [x] 🛡 Se há coleta de cookies <em>antes</em> de o usuário dar seu consentimento [^3]
- [x] 🔐 Se campos de criação de senha oferecem alguma checagem para garantir que a senha é forte e segura [^4]

## 🧭 Como usar

1. Acesse [https://data-sniffingcaramelo.onrender.com/](https://data-sniffingcaramelo.onrender.com/)
2. Insira a URL do website que deseja verificar
3. Clique em "Checar website" e aguarde enquanto o caramelo fareja a web para você!

> [!IMPORTANT]
> Ao inserir a URL, certifique-se de incluir o protocolo ("https://" ou "http://")

## 📌 Limitações conhecidas

- Por se basear no texto da página para encontrar tanto a política de privacidade quanto a opção de recusar coleta de cookies, é possível que o web crawler retorne um falso negativo caso o website analisado use termos diferentes daqueles previstos no código<br>
          <details>
                    <summary>
                              📑 Termos usados para encontrar a política de privacidade
                    </summary>
                    <ul>
                              <li>"política de privacidade"</li>
                              <li>"notificação de privacidade"</li>
                              <li>"privacy policy"</li>
                    </ul>
          </details>
          <details>
                    <summary>
                              🍪 Termos usados para encontrar a opção de recusar coleta de cookies
                    </summary>
                    <ul>
                              <li>"recusar"</li>
                              <li>"negar"</li>
                              <li>"não aceitar"</li>
                              <li>"rejeitar"</li>
                              <li>"refuse"</li>
                              <li>"reject"</li>
                    </ul>
          </details>

- Se o campo de senha for carregado depois do resto da página, é provável que o web crawler não o encontre

Encontrou mais algum problema ou limitação? Reporte na aba [Issues](https://github.com/mchd-nat/rust-undergrad-thesis/issues) do repositório.

## 📝 Licença
Esse projeto é licensiado sob a GNU General Public License v3.0 — veja o arquivo [LICENSE](./LICENSE) para detalhes.

[^1]: Baseada nos princípios do Livre Acesso, da Transparência e da Responsabilização e prestação de contas, previstos na Lei Geral de Proteção de Dados.
[^2]: Baseada no direito à informação sobre a possibilidade de não fornecer consentimento e sobre consequências da negativa, previsto na Lei Geral de Proteção de Dados.
[^3]: Baseada no princípio da Finalidade, previsto na Lei Geral de Proteção de Dados.
[^4]: Baseada no princípio da Segurança, previsto na Lei Geral de Proteção de Dados.

<br><p align="center">Copyright &copy; 2025-present <a href="https://mchd-nat.github.io/" target="_blank">Natália Silva Machado</a>
