#!/bin/bash
API_KEY="sk_live_myjkn036s5i-OqAQ8wptFE_9ji1_Wl-h86fGPUd2mUA"

echo "=== Test 6: Rate Limiting (send 65 requests rapidly) ==="
echo "Sending requests..."

for i in {1..65}; do
  RESPONSE=$(curl -s -w "\n%{http_code}" -X POST https://spell-platform.fly.dev/v1/cast \
    -H "Authorization: Bearer $API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"spell_name":"hello_spell","payload":{"message":"test"}}' | tail -1)

  if [ "$RESPONSE" = "429" ]; then
    echo ""
    echo "Request #$i: RATE LIMITED (429) ✓ Rate limiting works!"
    exit 0
  elif [ $i -le 60 ]; then
    printf "."
  else
    echo ""
    echo "Request #$i: HTTP $RESPONSE"
  fi
done
echo ""
