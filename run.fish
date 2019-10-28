#!/usr/bin/env fish

if not set -q RUST_LOG
    set RUST_LOG "info"
end

if test (count $argv) -lt 2
    echo usage: run.fish hostname test_case
else
    set name $argv[1]
    set tc $argv[2]

    docker run --rm --name $name --network proj2 -v (pwd)/results/tc$tc:/app/log -e RUST_LOG=$RUST_LOG prj2 -n $name -h hosts -t $tc -l log > results/tc$tc/$name.stdout.log 2> results/tc$tc/$name.stderr.log
end
