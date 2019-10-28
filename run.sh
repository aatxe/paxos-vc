#!/usr/bin/env sh

if [[ -z ${RUST_LOG} ]]
then
    RUST_LOG="info"
fi

if [[ $(count $argv) -lt 2 ]]
then
    echo usage: run.sh hostname test_case
else
    name=$argv[1]
    tc=$argv[2]

    docker run --rm --name $name --network proj2 -v $(pwd)/results/tc$tc:/app/log -e RUST_LOG=$RUST_LOG prj2 -n $name -h hosts -t $tc -l log > results/tc$tc/$name.stdout.log 2> results/tc$tc/$name.stderr.log
fi
