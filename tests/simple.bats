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

@test "--help includes some info about commands" {
  "$histo" --help
  "$histo" --help | grep -q "simple *Simple histogram"
  "$histo" --help | grep -q "select *Simple histogram of data selected .* by regex"
  "$histo" --help | grep -q "time-diff *.*distribution of difference betewen adjacent time stamps"
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

@test "simple select" {
  # TODO - check the output
  run "$histo" select "\((\d+\.\d+)\)" "$test_dir"/example.txt
  [ "$status" -eq 0 ]
}

@test "simple select with invalid filter" {
  run "$histo" select "\((\d+\.\u+)\)" "$test_dir"/example.txt
  [ "$status" -ne 0 ]
}

@test "simple select with no matching data" {
  run "$histo" select "\((\d+\.\d+\.)\)" "$test_dir"/example.txt
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
