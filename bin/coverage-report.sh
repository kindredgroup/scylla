#!/bin/bash
set -e

type rustup >/dev/null 2>&1 || { echo >&2 "rustup is not installed; aborting."; exit 1; }
type cargo >/dev/null 2>&1 || { echo >&2 "cargo is not installed; aborting."; exit 1; }
type grcov >/dev/null 2>&1 || { echo >&2 "grcov is not installed; aborting."; exit 1; }
type zip >/dev/null 2>&1 || { echo >&2 "zip is not installed; aborting."; exit 1; }

base_dir="$(dirname "$0")"
cd ${base_dir}/..
echo $base_dir
app_name=scylla
export CARGO_INCREMENTAL="0"
# export RUSTFLAGS="-C instrument-coverage"
export RUSTDOCFLAGS="-Cpanic=abort"
export RUSTFLAGS="-Cinstrument-coverage -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"

cargo update
echo "Compiling $app_name"
cargo +nightly build


echo "Testing $app_name"
export LLVM_PROFILE_FILE="${app_name}-%p-%m.profraw"
make withenv RECIPE=test.nightly

rm ccov.zip 2> /dev/null || true
zip -0 ccov.zip `find . \( -name "${app_name}*.gc*" \) -print`

echo "Generating HTML coverage report for $app_name"
rm -rf coverage 2> /dev/null || true
mkdir coverage
llvm-profdata merg -sparse **/*.profraw -o ${app_name}.profdata;
# grcov ccov.zip --branch -s . --llvm --ignore-not-existing --ignore "/*" --excl-start "\\\$coverage:ignore-start" --excl-stop "\\\$coverage:ignore-end" --excl-line "(//!|///|$coverage:ignore|unreachable!())" -t html -o coverage

echo "Generating LCOV coverage report for $app_name"
rm lcov.info 2> /dev/null || true
# grcov ccov.zip -s . --llvm  --ignore-not-existing --ignore "/*" --excl-start "\\\$coverage:ignore-start" --excl-stop "\\\$coverage:ignore-end" --excl-line "(//!|///|$coverage:ignore|unreachable!())" -t lcov -o lcov.info

# Clean up
# rm ccov.zip

make withenv RECIPE=test.json

if [ "$1" == "--open" ]; then
  index="file://$(pwd)/${base_dir}/../coverage/index.html"

  if command -v xdg-open &> /dev/null; then
    xdg-open $index
  elif command -v open &> /dev/null; then
    open $index
  else
    echo >&2 "neither xdg-open nor open are installed"
    exit 1
  fi
fi
