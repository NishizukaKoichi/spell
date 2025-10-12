#!/bin/bash
# Generate Software Bill of Materials (SBOM)
# Compliance: §9.4 - All spells must include SBOM (SPDX or CycloneDX)

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "==================================="
echo "SBOM Generation (§9.4 Compliance)"
echo "==================================="
echo ""

# Check if cargo-sbom is installed
if ! command -v cargo-sbom &> /dev/null; then
    echo -e "${YELLOW}cargo-sbom not found. Installing...${NC}"
    cargo install cargo-sbom
fi

# Output directory
SBOM_DIR="${SBOM_DIR:-./sbom}"
mkdir -p "$SBOM_DIR"

# Generate SPDX SBOM
echo "Generating SPDX SBOM..."
cargo sbom --output-format spdx_json_2_3 > "$SBOM_DIR/spell-api-sbom.spdx.json"
echo -e "${GREEN}✓ SPDX SBOM generated: $SBOM_DIR/spell-api-sbom.spdx.json${NC}"

# Generate CycloneDX SBOM
echo "Generating CycloneDX SBOM..."
cargo sbom --output-format cyclone_dx_json_1_4 > "$SBOM_DIR/spell-api-sbom.cdx.json"
echo -e "${GREEN}✓ CycloneDX SBOM generated: $SBOM_DIR/spell-api-sbom.cdx.json${NC}"

echo ""
echo "==================================="
echo "SBOM Generation Complete"
echo "==================================="
echo ""
echo "Files generated:"
ls -lh "$SBOM_DIR"
echo ""
echo "Next steps:"
echo "1. Review SBOM for accuracy"
echo "2. Scan for CVE vulnerabilities (cargo audit)"
echo "3. Sign SBOM with Sigstore (cosign)"
echo "4. Upload to artifact repository"
