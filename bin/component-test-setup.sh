#!/bin/bash
touch .env
echo PG_HOST=127.0.0.1 >> .env
echo PG_USER=admin >> .env
echo PG_PASSWORD=admin >> .env
echo PG_DATABASE=scylla >> .env
echo PG_PORT=5432 >> .env
# run db-migrations
make withenv RECIPE=db.migrate

 