set -e
package_name=$1
docker_file=Dockerfile.$package_name
version=$(cargo pkgid -p $package_name | cut -d# -f2 | cut -d: -f2)
package_version=$package_name:v$version
tag_ref=ghcr.io/kindredgroup/scylla/$package_version
echo $tag_ref
docker build -f $docker_file . --tag $tag_ref
#for ghcr.io access token mentioned in the github secrets and accessed in actions
docker push $tag_ref
