# Changelog

## [Unreleased]

### Features
  - (updater) suporte a downloads soberanos na banca e xdg updater (closes #18)
  - (ci) publicar release linux-x86_64 com tag latest na gambi (closes #20)
  - (meta) gerar card oficial de opengraph e banner social (closes #16)
  - (site) add documentation site, forgejo ci and cloudflare pages deploy
  - (milter) dual unix/tcp listener and smtp event loop (closes #11)
  - (config) hierarchical resolution and live hot-reload (closes #10)
  - (templates) native MJML via mrml and minijinja renderer (closes #9)
  - (signatures) long vs short signature replacement engine (closes #8)
  - (sanitizer) strip toxic HTML and isolate email content (closes #7)
  - (mime) parse and reconstruct MIME tree preserving attachments (closes #6)

### Bug Fixes
  - (filter) prevenir duplo encapsulamento e rastrear no emissor via cabecalho x-papeli (closes #23)
  - (lint) corrigir avisos de clippy que bloqueiam a CI (closes #22)
  - (build) configurar profile.release com LTO e strip, e crt-static musl (closes #19)
  - (milter) ignore SMFIC_MACRO without reply, support SMFIR_CHGHEADER and separate body payload
  - (milter) parse SMFIC_HEADER with null delimiters and insert EOH CRLF

## Previous Releases

