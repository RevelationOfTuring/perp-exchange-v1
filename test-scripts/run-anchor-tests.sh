#!/bin/sh
if [ "$1" = "all" ]; then
  test_files="tests/*.ts"
else
  test_files="$1"
fi

for test_file in $test_files; do
  export ANCHOR_TEST_FILE="$test_file"
  anchor test --skip-build || exit 1
done
