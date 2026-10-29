# Especificação Técnica: Sintaxe de Comentários no Cabeçalho HTTP do Site

- **Ticket**: #24
- **Escopo**: Correção da formatação de comentários em `site/_headers` para conformidade com a especificação de regras de cabeçalhos estáticos do Cloudflare Workers / Pages.

## 1. Contexto e Motivação
A publicação do site estático (`papeli.2lp.in`) via `wrangler deploy` falhou na esteira de CI com o erro `100324: Expected a colon-separated header pair` nas linhas iniciais de `site/_headers`.
O Cloudflare interpreta comentários exclusivamente iniciados pelo caractere cerquilha (`#`). Comentários formatados em estilo C (`/* ... */`) fazem com que o padrão `/*` seja identificado como seletor de rota global, interpretando o texto do comentário como pares de cabeçalho malformados.

## 2. Requisitos Técnicos
1. **Padronização dos Comentários em `site/_headers`**:
   - Todo e qualquer comentário informativo deve ser prefixado por `#`.
   - O seletor de rota global ou rotas específicas só devem conter pares válidos `Chave: Valor` ou linhas de comentário com `#`.

2. **Garantia de Conformidade GOST-09**:
   - Manter presentes todos os cabeçalhos obrigatórios de segurança (`X-Content-Type-Options: nosniff`, `X-Frame-Options: SAMEORIGIN`, `Referrer-Policy: strict-origin-when-cross-origin`).

3. **Verificação Automatizada**:
   - Adicionar teste de integridade/linter de sintaxe para `site/_headers`, assegurando que não contenha blocos com `/*` que não sejam rotas válidas seguidas de pares `Chave: Valor`.
