#!/bin/sh
for snapshot in lib/src/tests/snapshots/*; do
  cargo insta show "$snapshot"
  echo 'Press enter to continue'
  read -r _
done
