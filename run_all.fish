#!/usr/bin/env fish

mkdir -p results/tc1 results/tc2 results/tc3 results/tc4 results/tc5

if not set -q RUST_LOG
    set RUST_LOG "info"
end

for n in (seq 5)
    echo BEGIN $argv TEST $n
    docker run --rm --name $argv --network proj2 -v (pwd)/results/tc$n:/app/log -e RUST_LOG=$RUST_LOG prj2 -n $argv -h hosts -t $n -l log > results/tc$n/$argv.stdout.log 2> results/tc$n/$argv.stderr.log
    echo END $argv TEST $n
    sleep 30
end
