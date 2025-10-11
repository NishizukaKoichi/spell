#!/bin/bash
set -euo pipefail

# Apply database migrations via Fly.io proxy
# Usage: ./scripts/apply_migrations.sh

echo "Starting database migration..."

# Check if psql is available
if ! command -v psql &> /dev/null; then
    echo "Error: psql is not installed"
    echo "On macOS: brew install libpq"
    echo "Then add to PATH: export PATH=\"/opt/homebrew/opt/libpq/bin:\$PATH\""
    exit 1
fi

# Check if flyctl is available
if ! command -v flyctl &> /dev/null; then
    echo "Error: flyctl is not installed"
    echo "Install: curl -L https://fly.io/install.sh | sh"
    exit 1
fi

echo "Getting DATABASE_URL from production..."
DATABASE_URL=$(flyctl ssh console -a spell-platform -C "printenv DATABASE_URL" 2>/dev/null || echo "")

if [ -z "$DATABASE_URL" ]; then
    echo "Error: Could not get DATABASE_URL from production"
    echo "Attempting alternative method with proxy..."

    # Start proxy in background
    echo "Starting database proxy on localhost:15432..."
    flyctl proxy 15432:5432 -a spell-platform-db &
    PROXY_PID=$!

    # Wait for proxy to be ready
    sleep 5

    # Trap to ensure proxy is killed on exit
    trap "kill $PROXY_PID 2>/dev/null || true" EXIT

    echo "Applying migrations via proxy..."

    # Try to get connection details from app
    DB_USER=$(flyctl ssh console -a spell-platform -C "printenv DATABASE_URL | cut -d/ -f3 | cut -d@ -f1 | cut -d: -f1" 2>/dev/null || echo "spell_platform")
    DB_NAME=$(flyctl ssh console -a spell-platform -C "printenv DATABASE_URL | rev | cut -d/ -f1 | rev" 2>/dev/null || echo "spell_platform")

    echo "Using connection: postgresql://$DB_USER@localhost:15432/$DB_NAME"
    echo "Note: You may need to set PGPASSWORD environment variable"

    # Apply each migration
    for migration in migrations/*.sql; do
        echo "Applying $migration..."
        if psql -h localhost -p 15432 -U "$DB_USER" -d "$DB_NAME" -f "$migration"; then
            echo "✓ $migration applied successfully"
        else
            echo "✗ Failed to apply $migration (may already be applied)"
        fi
    done

    # Kill proxy
    kill $PROXY_PID 2>/dev/null || true
else
    echo "Applying migrations directly..."

    # Apply each migration
    for migration in migrations/*.sql; do
        echo "Applying $migration..."
        if psql "$DATABASE_URL" -f "$migration"; then
            echo "✓ $migration applied successfully"
        else
            echo "✗ Failed to apply $migration (may already be applied)"
        fi
    done
fi

echo ""
echo "Migration process completed!"
echo ""
echo "Verifying billing tables..."
flyctl ssh console -a spell-platform -C "psql \$DATABASE_URL -c '\\dt billing*'" || true

echo ""
echo "Done!"
