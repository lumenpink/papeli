#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HEADERS_FILE="$REPO_ROOT/site/_headers"
SPEC_FILE="$REPO_ROOT/docs/specs/24_site_headers_syntax.md"

FAILED=0

echo "==> [TEST] Validando conformidade sintática de site/_headers (#24)..."

# 1. Spec presente
if [ ! -f "$SPEC_FILE" ]; then
    echo "FALHA: $SPEC_FILE não encontrado."
    FAILED=1
else
    echo "OK: $SPEC_FILE presente."
fi

# 2. Arquivo _headers presente
if [ ! -f "$HEADERS_FILE" ]; then
    echo "FALHA: $HEADERS_FILE não encontrado."
    exit 1
fi

# 3. Não conter comentários em estilo C (/* ... */) que quebram o parser do Cloudflare
if grep -q -E '^[[:space:]]*\*[[:space:]]' "$HEADERS_FILE" || grep -q '\*/' "$HEADERS_FILE"; then
    echo "FALHA: Comentários no estilo C detectados em $HEADERS_FILE. Cloudflare requer '#'."
    FAILED=1
else
    echo "OK: Nenhum comentário em estilo C detectado."
fi

# 4. Validar que toda linha de rota ou header seja válida
CURRENT_ROUTE=""
LINE_NUM=0
while IFS= read -r line || [ -n "$line" ]; do
    LINE_NUM=$((LINE_NUM + 1))
    # Ignorar linhas vazias e comentários com #
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue

    if [[ "$line" =~ ^/ ]]; then
        CURRENT_ROUTE="$line"
    elif [[ "$line" =~ ^[[:space:]]+([A-Za-z0-9-]+):[[:space:]]* ]]; then
        # Par válido de cabeçalho
        continue
    else
        echo "FALHA: Linha $LINE_NUM inválida para o Cloudflare _headers: '$line'"
        FAILED=1
    fi
done < "$HEADERS_FILE"

# 5. Validar cabeçalhos de segurança obrigatórios
for H in "X-Content-Type-Options" "X-Frame-Options" "Referrer-Policy"; do
    if ! grep -q "$H" "$HEADERS_FILE"; then
        echo "FALHA: Cabeçalho obrigatório '$H' ausente em $HEADERS_FILE."
        FAILED=1
    else
        echo "OK: Cabeçalho '$H' presente."
    fi
done

if [ $FAILED -ne 0 ]; then
    echo "==> [TEST] STATUS: RED (Asserções falharam conforme esperado)"
    exit 1
fi

echo "==> [TEST] STATUS: GREEN (site/_headers sintaticamente conforme)"
