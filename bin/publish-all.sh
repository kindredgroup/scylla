#!/bin/bash
set -e

type cargo-get >/dev/null 2>&1 || { echo >&2 "cargo-get is not installed; aborting."; exit 1; }

project="sequent"
base_dir="$(dirname "$0")"
cd ${base_dir}/..
proj_dir=$(pwd)
new_version=$(cd $project; cargo get version)

function await() {
  package=$1
  echo -n "Awaiting indexing of package ${package}"

  # create a new, dummy project, and try to add the recently published package as a dependency
  temp_dir=${TMPDIR-/tmp}
  cd $temp_dir
  dummy_proj="${package}-dummy"
  rm -rf $dummy_proj 2> /dev/null || true
  cargo new -q $dummy_proj
  cd $dummy_proj

  while [ true ]; do
    has_no_version=$(cargo add ${package}@${new_version} 2>&1 | grep "could not be found" | wc -l)
    if [ "${has_no_version}" -eq "1" ]; then
      echo -n "."
      sleep 2
    else
      echo
      cd ..
      rm -rf $dummy_proj
      cd $proj_dir
      break
    fi

  done
}

function publish() {
  package=$1
  cargo publish -p $package
  await $package
}

echo "Publishing all packages for $project ${new_version}"

# publish sequent
# publish sequent-repl