#!/bin/bash
set -e
version=$(cargo pkgid -p scylla_pg_monitor | cut -d# -f2 | cut -d: -f2)
package_version=scylla_pg_monitor:$version
docker build . -t $package_version
docker tag $package_version anil0906/$package_version
echo $DOCKER_HUB_TOKEN | docker login -u $DOCKER_HUB_USER --password-stdin
docker push anil0906/$package_version
