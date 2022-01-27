#!/usr/bin/env bash
set -x
set -eo pipefail

if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run \
  -p 9000:9000 \
  -p 9001:9001 \
  minio/minio server /data --console-address ":9001"
fi
# ^ Increased maximum number of connections for testing purposes
# Keep pinging Postgres until it's ready to accept commands

>&2 echo "Minio ready to go"