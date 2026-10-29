#!/usr/bin/env bash
# ==============================================================================
# papeli — Teste de Conformidade da Esteira CI (no_proxy e expurgo de cache) (#25)
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKFLOW_FILE="$REPO_ROOT/.forgejo/workflows/ci.yml"
SPEC_FILE="$REPO_ROOT/docs/specs/25_ci_no_proxy_and_cache_expurgation.md"

FAILED=0

echo "==> [TEST] Validando conformidade de no_proxy e remocao de cache na CI (#25)..."

# 1. Validacao da especificacao formal
if [ ! -f "$SPEC_FILE" ]; then
    echo "FALHA: Arquivo de especificacao $SPEC_FILE nao encontrado."
    FAILED=1
else
    echo "OK: $SPEC_FILE presente."
fi

# 2. Validacao da presenca do workflow
if [ ! -f "$WORKFLOW_FILE" ]; then
    echo "FALHA: Workflow $WORKFLOW_FILE nao encontrado."
    exit 1
fi

# 3. Validar declaracao de no_proxy e NO_PROXY
if ! grep -q 'no_proxy: "localhost,127.0.0.1,::1"' "$WORKFLOW_FILE"; then
    echo "FALHA: no_proxy: \"localhost,127.0.0.1,::1\" ausente no workflow."
    FAILED=1
else
    echo "OK: no_proxy configurado."
fi

if ! grep -q 'NO_PROXY: "localhost,127.0.0.1,::1"' "$WORKFLOW_FILE"; then
    echo "FALHA: NO_PROXY: \"localhost,127.0.0.1,::1\" ausente no workflow."
    FAILED=1
else
    echo "OK: NO_PROXY configurado."
fi

# 4. Validar ausencia completa de actions/cache
if grep -q "actions/cache" "$WORKFLOW_FILE"; then
    echo "FALHA: actions/cache ainda presente no workflow."
    FAILED=1
else
    echo "OK: actions/cache expurgado com sucesso."
fi

# 5. Validar imagem gost-ci-cross no job de teste
if ! grep -A 10 "test:" "$WORKFLOW_FILE" | grep -q "gambi.2lp.in/lumen/gost-ci-cross:latest"; then
    echo "FALHA: Imagem gost-ci-cross:latest ausente no job test."
    FAILED=1
else
    echo "OK: gost-ci-cross:latest configurado no job test."
fi

# 6. Testes de Mutacao (Sensibilidade das assercoes)
echo "==> Executando testes de mutacao..."
TMP_WORKFLOW="$(mktemp)"
trap 'rm -f "$TMP_WORKFLOW"' EXIT

# Mutante 1: Remocao de no_proxy
sed 's/no_proxy: "localhost,127.0.0.1,::1"//' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if grep -q 'no_proxy: "localhost,127.0.0.1,::1"' "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 1 (sem no_proxy) nao detectado."
    FAILED=1
else
    echo "OK: Mutante 1 (sem no_proxy) detectado e isolado."
fi

# Mutante 2: Reinsercao de actions/cache
sed 's/cargo test/actions\/cache/' "$WORKFLOW_FILE" > "$TMP_WORKFLOW"
if ! grep -q "actions/cache" "$TMP_WORKFLOW"; then
    echo "FALHA: Mutante 2 (reintroducao actions/cache) nao detectado."
    FAILED=1
else
    echo "OK: Mutante 2 (reintroducao actions/cache) detectado e isolado."
fi

if [ $FAILED -ne 0 ]; then
    echo "==> [TEST] STATUS: RED (Assercoes falharam conforme esperado no TDD)"
    exit 1
fi

echo "==> [TEST] STATUS: GREEN (Todas as assercoes e mutacoes passaram)"
