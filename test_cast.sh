#!/bin/bash
curl -X POST https://spell-platform.fly.dev/v1/cast \
  -H "Content-Type: application/json" \
  -d '{"spell_name":"hello_spell","payload":{"message":"Hello from Spell Platform!"}}'
