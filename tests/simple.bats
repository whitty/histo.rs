#!/usr/bin/env bats

setup_file() {
  # ensure we have up to date build
  cargo build
}

setup() {
  # ensure executable exists
  histo=target/debug/histo-log
  test -x "$histo"

  test_dir=tests

  # Set a fixed COLUMNS value for tests
  export COLUMNS=72
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
  run "$histo" time-diff --time-delta=200 "$test_dir"/example.txt
  [ "$status" -eq 0 ]
  [ "$output" = "     200 #################################################
     400 ##############################################################
     600 ##################
     800 ########################
    1000 ######
    1200 ######
    1400 ######
    1600
    1800
    2000 ######
    2200
    2400
    2600
    2800
    3000 ######" ]
}

@test "time-diff with select" {
  # TODO - check the output - I don't think its very good
  run "$histo" time-diff --time-select='\((\d+\.\d+)\)' "$test_dir"/example.txt
  [ "$status" -eq 0 ]
}

@test "scoped: recurse" {
  # TODO - check the output - I don't think its very good
  "$histo" scoped --scope-in="->recurse" --scope-out="<-recurse" "$test_dir"/example_scoped.txt
}

@test "scoped: recurse --count --time-delta" {
  run "$histo" scoped --show-counts --time-delta=1000 --scope-in="->recurse" --scope-out="<-recurse" "$test_dir"/example_scoped.txt
  [ "$status" -eq 0 ]
  [ "$output" = "    1000: 2 ###########################################################
    2000: 0
    3000: 0
    4000: 1 #############################
    5000: 0
    6000: 1 #############################" ]
}

@test "scoped: arg conflict" {
  run "$histo" scoped --scope-in="->recurse" --scope-match="<-recurse" "$test_dir"/example_scoped.txt
  [ "$status" -ne 0 ]
  echo "$output" | grep -q "scope-in .* cannot be used with.*scope-match"
}

@test "scoped: missing required arg" {
  # TODO - check the output - I don't think its very good
  run "$histo" scoped --scope-in="->recurse" "$test_dir"/example_scoped.txt
  [ "$status" -ne 0 ]
  echo "$output" | grep -q "The following required arguments were not provided"
}

@test "scoped: no match fails" {
  run "$histo" scoped --scope-in="->output" --scope-out="<-input" "$test_dir"/example_scoped.txt
  [ "$status" -ne 0 ]
  echo "$output" | grep -q "No data found"
}

# TODO - add helpers for strace mode
@test "scoped: strace" {
  run "$histo" scoped --show-counts --time-delta=0.0002 --time-select="\d+:\d+:(\d+\.\d+)" --scope-in="openat\(.*\) = (\d)" --scope-out="close\((\d)\)" "$test_dir"/strace.txt
  [ "$status" -eq 0 ]
  [ "$output" = "  0.0002: 1 ###################
  0.0004: 1 ###################
  0.0006: 3 ###########################################################
  0.0008: 1 ###################" ]
}
