#!/bin/bash

#script 
if [ "${SCYLLA_CLEANUP_MODE}" == "true" ];
then
  echo "Cleanup mode is ON"
  env | grep "PG_DATABASE"
  /opt/build/db_delete
else
  if [ "${SCYLLA_CREATE_DB}" == "true" ];
  then
    echo "Running 'db_create'"
    /opt/build/db_create
    sleep 3
  fi

  echo "Running 'db_migrate'"
  /opt/build/db_migrate 
fi