#!/bin/bash

#load env variables

# run db-migrations
sh cargo run --bin db_migrate
 