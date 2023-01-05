#!/bin/bash
set -e
target=$1
current_directory=$PWD
# check if version matches for package.json and cargo.toml
cargo_version=$(cargo pkgid -p scylla_pg_js | cut -d# -f2 | cut -d: -f2)
echo $cargo_version
# updating same version for all the related packages

set_version() {
  echo  "updated version for $1"
  npm --prefix ./$1 pkg set version=$cargo_version
}
set_version ./scylla_pg_js/
set_version ./scylla_pg_client/
#build scylla_pg_js
cd scylla_pg_js
npm install
npm run build --target "artifacts/$target"
npm run artifacts
# update versions for multiple arch
cd npm
arch_packages=$(find . -maxdepth 1 -type d \( ! -name . \))
for i in $arch_packages; do
    set_version "$i"
    npm prefix
done

#release_package() {
#  cd $current_directory
#  package_name=$1
#  target=$2
#  echo "npm install for package name $package_name and target $target"
#  npm --prefix $package_name install
#  echo "npm run build for package name $package_name and target $target"
#  npm --prefix $package_name run build --target $target
#  echo "npm publish for package name $package_name and target $target"
#  cd $package_name
#  npm publish
#}




# release scylla_pg_js
#release_package scylla_pg_js $artifact_path
## release scylla_pg_client
#release_package scylla_pg_client .



