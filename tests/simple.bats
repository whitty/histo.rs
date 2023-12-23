#!/usr/bin/env bats

setup_file() {
  # ensure we have up to date build
  cargo build
}

setup() {
  # ensure executable exists
  histo=target/debug/histo
  test -x "$histo"

  test_dir=tests
}

@test "no args fails" {
  ! "$histo"
}

@test "--help doesn't fail" {
  "$histo" --help
}

@test "simple" {
  # TODO - check the output - I don't think its very good
  run "$histo" simple "$test_dir"/seq.txt
  [ "$status" -eq 0 ]
}

@test "simple with filter" {
  # TODO - check the output - I don't think its very good
  run "$histo" simple --match 2 "$test_dir"/seq.txt
  [ "$status" -eq 0 ]
}

@test "simple with invalid filter" {
  run "$histo" simple --match "\u" "$test_dir"/seq.txt
  [ "$status" -ne 0 ]
}

@test "time-diff simple" {
  # TODO - check the output - I don't think its very good
  run "$histo" time-diff "$test_dir"/example.txt
  [ "$status" -eq 0 ]
}

@test "time-diff with select" {
  # TODO - check the output - I don't think its very good
  run "$histo" time-diff --time-select='\((\d+\.\d+)\)' "$test_dir"/example.txt
  [ "$status" -eq 0 ]
}
