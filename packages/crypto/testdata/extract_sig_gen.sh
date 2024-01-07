#!/bin/bash

SIGGEN="$1"
[ -z "$SIGGEN" ] && echo "Usage: $0 <NIST SigGen>.txt [CURVE] [HASH]" && echo "Eg.: $0 186-4ecdsatestvectors/SigGen.txt P-256 SHA-224" && exit 1
CURVE="$2"
[ -z "$CURVE" ] && CURVE="P-256"
HASH="$3"
[ -z "$HASH" ] && HASH="SHA-256"

cat $SIGGEN | sed 's///' | \
sed -n -E "/^\[$CURVE,$HASH\]/,/^\[/{s/\[$CURVE,$HASH\]/\[/;s/^Msg = *([^ ]*)/  {\n    \"message\": \"\1\",/;/^Qx =/{N;s/^Qx = *([^ ]*)\nQy = *([^ ]*)/    \"pubkey\": \"04\1\2\",/};/^R =/{N;s/^R = *([^ ]*)\nS = *([^ ]*)/    \"signature\": \"\1\2\"\n\  },/};s/^\[.*\]/\]/;p}" | \
grep -E '[[{}"]|]' | \
tac | \
sed '2s/},/}/' | \
tac
