#!/bin/bash
set -e
current_directory=$PWD
rm -rf Cargo.lock
#cargo build --release
cargo_version=$(cargo pkgid -p scylla_pg_js | cut -d# -f2 | cut -d: -f2)
echo $cargo_version
# updating same version for all the related packages

set_version() {
  echo  "updating version for $1"
  npm --prefix ./$1 pkg set version=$cargo_version
}



# update versions for multiple arch
release_package() {
  package_name=$1
  cd $package_name
  npm run prepublishonly
  echo "//registry.npmjs.org/:_authToken=$NPM_TOKEN" >> ~/.npmrc
  npm publish
}


#arch_packages=$(find scylla_pg_js/npm -maxdepth 1 -type d \( ! -name npm \))
#for i in $arch_packages; do
#    cd $current_directory
#    set_version "$i"
#    release_package "$i"
#done

# release pg_js package
cd $current_directory
set_version ./scylla_pg_js/
release_package scylla_pg_js
# release pg_client package
cd $current_directory
set_version ./scylla_pg_client/
echo " curent: $PWD"

npm --prefix ./scylla_pg_client/ pkg set dependencies.scylla_pg_js=$cargo_version
npm --prefix ./scylla_pg_client/ install
npm --prefix ./scylla_pg_client/ run build

release_package scylla_pg_client




