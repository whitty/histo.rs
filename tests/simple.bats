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
  "$histo" --help | sed '/simple/ {N;s/\n//'} | grep -q "simple *Simple histogram"
  "$histo" --help | sed '/select/ {N;s/\n//'} | grep -q "select *Simple histogram of data selected .* by regex"
  "$histo" --help | sed '/time-diff/ {N;s/\n//'} | grep -q "time-diff *.*distribution of difference betewen adjacent time stamps"
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

@test "simple select with no capture" {
  run "$histo" select ".+" "$test_dir"/example.txt
  [ "$status" -ne 0 ]
  echo "$output" | grep "Need at least one regex match"
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

@test "scoped recuse" {
  # TODO - check the output - I don't think its very good
  "$histo" scoped --scope-in="->recurse" --scope-out="<-recurse" "$test_dir"/example_scoped.txt
}

@test "scoped arg conflict" {
  # TODO - check the output - I don't think its very good
  run "$histo" scoped --scope-in="->recurse" --scope-match="<-recurse" "$test_dir"/example_scoped.txt
  [ "$status" -ne 0 ]
}

@test "scoped arg requires" {
  # TODO - check the output - I don't think its very good
  run "$histo" scoped --scope-in="->recurse" "$test_dir"/example_scoped.txt
  [ "$status" -ne 0 ]
}
