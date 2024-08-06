[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/z3oDTWtZ)
[![Open in Codespaces](https://classroom.github.com/assets/launch-codespace-2972f46106e565e64193e422d61a12cf1da4916b45550586e14ef0a7c637dd04.svg)](https://classroom.github.com/open-in-codespaces?assignment_repo_id=15435869)

# FSE 2024.1 - Trabalho 2

## Elevadores

## Sobre o projeto

O trabalho foi feito inteiramente na linguagem Rust e foi testado em uma raspberry pi com o sistema linux. Para executa-lo, é necessário instalar o ambiente de desenvolvimento Rust, compilar o projeto (normalmente com o comando `cargo build`) e executar o binário gerado na pasta `target`.

## Pré-requisitos para a compilação

- Rustup (rustc, cargo, etc) instalado e configurado no PATH (https://www.rust-lang.org/tools/install)
    - Sempre necessário para compilar o projeto
- Cross (https://github.com/cross-rs/cross)
  - É necessário para fazer cross-compilação para a raspberry pi, caso não deseje compilar diretamente no target (recomendado)
  - Docker (https://docs.docker.com/get-docker/)
    - Pré-requisito para o cross

## Como compilar

### Compilando diretamente no target

1. envie o repositório para a raspberry pi
2. execute o comando `cargo build --release` na pasta do projeto.
3. o binário estará na pasta `target/release`, basta executar.

### Cross-compilando (recomendado)

1. execute o comando `cross build --release --target armv7-unknown-linux-musleabihf` na pasta do projeto.

2. o binário estará na pasta target/armv7-unknown-linux-musleabihf/release, envie o binário para a raspberry pi da forma que preferir, scp por exemplo.

3. execute o binário correspondente na raspberry pi.

> **IMPORTANTE**: Para executar o binário na raspberry pi, é necessário que o binário tenha permissão de execução. Caso não tenha, execute o comando `chmod 744 <nome_do_binario>`.

## Vídeos de demonstração
- Demonstração da compilação e das funcionalidades: (https://youtu.be/1Ppof8FnLjc)

## Experimento

Enviando elevador 1 para o andar 3 e elevador 2 para o andar 1.

![Experimento](./assets/experimento.png)