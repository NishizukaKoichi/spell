#!/bin/bash
# Sign artifacts with Sigstore (Fulcio + Rekor)
# Compliance: §9.2 - Supply chain verification with Sigstore

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "==================================="
echo "Sigstore Signing (§9.2 Compliance)"
echo "==================================="
echo ""

# Check if cosign is installed
if ! command -v cosign &> /dev/null; then
    echo -e "${RED}cosign not found.${NC}"
    echo "Install cosign:"
    echo "  brew install cosign  # macOS"
    echo "  or visit: https://github.com/sigstore/cosign"
    exit 1
fi

# Artifacts to sign
ARTIFACT="${1:-}"
SBOM_DIR="${SBOM_DIR:-./sbom}"

if [ -z "$ARTIFACT" ]; then
    echo "Usage: $0 <artifact_path>"
    echo ""
    echo "Example:"
    echo "  $0 ./target/release/spell-api"
    echo "  $0 ./sbom/spell-api-sbom.spdx.json"
    exit 1
fi

if [ ! -f "$ARTIFACT" ]; then
    echo -e "${RED}Artifact not found: $ARTIFACT${NC}"
    exit 1
fi

echo "Signing artifact: $ARTIFACT"
echo ""

# Sign with Fulcio (keyless signing)
echo "Signing with Fulcio (keyless)..."
cosign sign-blob "$ARTIFACT" \
    --output-signature="${ARTIFACT}.sig" \
    --output-certificate="${ARTIFACT}.pem"

echo -e "${GREEN}✓ Signature: ${ARTIFACT}.sig${NC}"
echo -e "${GREEN}✓ Certificate: ${ARTIFACT}.pem${NC}"

# Verify transparency log (Rekor)
echo ""
echo "Verifying Rekor transparency log..."
cosign verify-blob "$ARTIFACT" \
    --signature="${ARTIFACT}.sig" \
    --certificate="${ARTIFACT}.pem" \
    --certificate-identity-regexp=".*" \
    --certificate-oidc-issuer-regexp=".*"

echo ""
echo "==================================="
echo "Signing Complete"
echo "==================================="
echo ""
echo "Files generated:"
ls -lh "${ARTIFACT}"*
echo ""
echo "Verification:"
echo "  cosign verify-blob $ARTIFACT \\"
echo "    --signature=${ARTIFACT}.sig \\"
echo "    --certificate=${ARTIFACT}.pem \\"
echo "    --certificate-identity-regexp='.*' \\"
echo "    --certificate-oidc-issuer-regexp='.*'"
