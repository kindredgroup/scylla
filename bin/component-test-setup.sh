#!/bin/bash

# run db-migrations
PGUSER=admin PGPASSWORD=admin PGDATABASE=scylla PGHOST=127.0.0.1 PGPORT=5432 cargo run --bin db_migrate cargo run --bin db_migrate

# Seed data or call test script
 