# Data-Sniffing Caramelo

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white) ![HTML5](https://img.shields.io/badge/html5-%23E34F26.svg?style=for-the-badge&logo=html5&logoColor=white) ![CSS3](https://img.shields.io/badge/css3-%231572B6.svg?style=for-the-badge&logo=css3&logoColor=white) ![JavaScript](https://img.shields.io/badge/javascript-%23323330.svg?style=for-the-badge&logo=javascript&logoColor=%23F7DF1E) ![Render](https://img.shields.io/badge/Render-%46E3B7.svg?style=for-the-badge&logo=render&logoColor=white)

## Sobre a ferramenta

*Data-Sniffing Caramelo* é seu cão farejador virtual que inspeciona sites da web para encontrar infrações à Lei Geral de Proteção de Dados.

A ferramenta foi criada como parte do meu trabalho de conclusão do curso de Ciência da Computação e ainda está em desenvolvimento. No momento, ela é capaz de fazer quatro checagens:
- Presença (ou ausência) de uma política de privacidade;
- Presença (ou ausência) de aviso sobre a coleta de cookies e a opção de recusar tal coleta;
- Se há coleta de cookies <em>antes</em> de o usuário dar seu consentimento;
- Se campos de senha (como em formulários de cadastro) oferecem alguma checagem para garantir que a senha escolhida pelo usuário é forte e segura.

O *web crawler* em si é desenvolvido em Rust. JavaScript é usado para manipular os elementos HTML e fazer a conexão entre o front-end e o programa em Rust.
          
