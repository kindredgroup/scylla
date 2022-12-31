#!/bin/bash
set -e
# check if version matches for package.json and cargo.toml
cargo_version=$(cargo pkgid -p scylla_pg_js | cut -d# -f2 | cut -d: -f2)
echo $cargo_version
#cd scylla_pg_js
package_version=$(npm --prefix ./scylla_pg_js/ pkg get version)
echo $package_version
if [ $package_version != "\"$cargo_version\"" ]
then
  echo "version mismatched"
else
  rm -rf scylla_pg_client
  mkdir scylla_pg_client
  cd scylla_pg_client
  package_path=../scylla_pg_js
  # copy required files to new folder
  cp $package_path/package.json $package_path/type.d.ts $package_path/index.d.ts $package_path/index.cjs $package_path/scylla.node $package_path/.npmrc $package_path/README.md .
  npm publish
fi
#cd ..


