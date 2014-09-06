#!/bin/bash

echo "Checking if any rust file has a line longer than 99 characters"

offenders=$(grep -Pl ".{100}" $(find . -name '*.rs'))

if [[ ! -z $offenders ]]; then
  for offender in $offenders; do
    echo "> $offender exceeds 99 chars"
    awk 'length($0) > 99' $offender
  done

  exit 1
fi

echo "All is good!"
