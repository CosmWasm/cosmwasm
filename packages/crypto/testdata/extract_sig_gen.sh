#!/usr/bin/env bash

function usage() {
  echo "Usage: $0 [-c|--curves] [-h|--hashes] <NIST SigGen>.txt [CURVE] [HASH]"
  echo "Eg.: $0 186-4ecdsatestvectors/SigGen.txt P-256 SHA-224"
  exit 1
}

CURVES=0
HASHES=0
while [[ $# -gt 0 ]]; do
  KEY="$1"
  case $KEY in
  -c | --curves)
    CURVES=1
    shift
    ;;
  -h | --hashes)
    HASHES=1
    shift
    ;;
  *)
    break
    ;;
  esac
done

SIGGEN="$1"
[ -z "$SIGGEN" ] && usage
[ $CURVES -eq 1 ] && grep '^\[' "$SIGGEN" | cut -f2 -d[ | cut -f1 -d, | sort -u
[ $HASHES -eq 1 ] && grep '^\[' "$SIGGEN" | cut -f1 -d] | cut -f2 -d, | sort -u
[ $CURVES -eq 1 ] || [ $HASHES -eq 1 ] && exit 0

CURVE="$2"
[ -z "$CURVE" ] && CURVE="P-256"
HASH="$3"
[ -z "$HASH" ] && HASH="SHA-256"

cat $SIGGEN |
  sed 's/\r//' |
  sed -n -E "/^\[$CURVE,$HASH\]/,/^\[/{s/\[$CURVE,$HASH\]/\[/;s/^Msg = *([^ ]*)/  {\n    \"message\": \"\1\",/;/^Qx =/{N;s/^Qx = *([^ ]*)\nQy = *([^ ]*)/    \"pubkey\": \"04\1\2\",/};/^R =/{N;s/^R = *([^ ]*)\nS = *([^ ]*)/    \"signature\": \"\1\2\"\n\  },/};s/^\[.*\]/\]/;p}" |
  grep -E '[[{}"]|]' |
  tac |
  sed '2s/},/}/' |
  tac
