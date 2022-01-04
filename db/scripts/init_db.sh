#!/usr/bin/env bash
set -x
set -eo pipefail

>&2 echo "Postgres is up and running on port  - running migrations now!"

sqlx database create
sqlx migrate run

>&2 echo "MariaDB has been migrated, ready to go!"