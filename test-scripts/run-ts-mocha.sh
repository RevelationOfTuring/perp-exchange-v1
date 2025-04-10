#!/bin/sh
yarn run ts-mocha -p ./tsconfig.json -t 1000000 ${ANCHOR_TEST_FILE}