#!/bin/bash
set -e
package_name=$1
tag=$2
if [ -z "$tag" ]; then
   tag="$package_name"
fi
docker_file=Dockerfile.$tag
cargo pkgid -p $package_name # to check if the package name is correct
version=$(cargo pkgid -p $package_name | cut -d# -f2 | cut -d: -f2)
tag_with_version=$tag:v$version
tag_ref=ghcr.io/kindredgroup/scylla/$tag_with_version
echo $tag_ref
docker build -f $docker_file . --tag $tag_ref
#for ghcr.io access token mentioned in the github secrets and accessed in actions
docker push $tag_ref
