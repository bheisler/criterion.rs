#!/bin/bash

echo "Checking if any rust file has a line longer than 79 characters"

offenders=$(grep -Pl ".{80}" $(find . -name '*.rs'))
status=$?

if [[ $status == 0 ]]; then
  for offender in $offenders; do
    echo "> $offender exceeds 79 chars"
    awk 'length($0) > 79' $offender
  done

  exit 1
fi

echo "All is good!"
