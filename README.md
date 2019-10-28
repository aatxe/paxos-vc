# paxos-vc

paxos-vc is an implementation of the view change (also known as leader election) portion of the Paxos protocol in Rust by [Aaron Weiss](https://aaronweiss.us). This document contains instructions for running the system in Docker containers. It requires you to have created a network named proj2 using `docker network create --driver bridge proj2`. I have also included results (stdout, stderr, and detailed logs) from every node during the execution of every test case. These can be reproduced with the included scripts described in the rest of this document.

## Test Cases for Fish Users

To run the test cases with fish, there are two options. You can either use the `run.fish` script to run a specific host and test case by name (e.g. `run.fish columba 2`) to run the columba container for test case 2, or you can use `run_all.fish` with simply the name of the host (e.g. `run_all.fish columba`) to run through all the trials in sequence. The latter process is more automated, but takes more time as it spends thirty seconds waiting between trials to ensure that everyone has terminated. Since test case 5 does not terminate, you will have to kill the containers directly using `docker kill columba raphus turtur geopelia trogon` if you would like to end its execution.

## Test Cases for POSIX Shell Users

I don't personally use the POSIX shell directly, and so I can't guarantee that they work, but I have attempted to port the same fish scripts to bash. Alternatively, if they don't work, you can find subsections below that include the correct commands to invoke for each container.

### Test Case 1

```
# run server 0 (columba)
docker run --rm --name columba --network proj2 -v $(pwd)/results/tc1:/app/log -e RUST_LOG=trace prj2 -n columba -h hosts -t 1 -l log > results/tc1/columba.stdout.log 2> results/tc1/columba.stderr.log

# run server 1 (raphus)
docker run --rm --name raphus --network proj2 -v $(pwd)/results/tc1:/app/log -e RUST_LOG=trace prj2 -n raphus -h hosts -t 1 -l log > results/tc1/raphus.stdout.log 2> results/tc1/raphus.stderr.log

# run server 2 (turtur)
docker run --rm --name turtur --network proj2 -v $(pwd)/results/tc1:/app/log -e RUST_LOG=trace prj2 -n turtur -h hosts -t 1 -l log > results/tc1/turtur.stdout.log 2> results/tc1/turtur.stderr.log

# run server 3 (geopelia)
docker run --rm --name geopelia --network proj2 -v $(pwd)/results/tc1:/app/log -e RUST_LOG=trace prj2 -n geopelia -h hosts -t 1 -l log > results/tc1/geopelia.stdout.log 2> results/tc1/geopelia.stderr.log

# run server 4 (trogon)
docker run --rm --name trogon --network proj2 -v $(pwd)/results/tc1:/app/log -e RUST_LOG=trace prj2 -n trogon -h hosts -t 1 -l log > results/tc1/trogon.stdout.log 2> results/tc1/trogon.stderr.log
```

### Test Case 2

```
# run server 0 (columba)
docker run --rm --name columba --network proj2 -v $(pwd)/results/tc2:/app/log -e RUST_LOG=trace prj2 -n columba -h hosts -t 2 -l log > results/tc2/columba.stdout.log 2> results/tc2/columba.stderr.log

# run server 1 (raphus)
docker run --rm --name raphus --network proj2 -v $(pwd)/results/tc2:/app/log -e RUST_LOG=trace prj2 -n raphus -h hosts -t 2 -l log > results/tc2/raphus.stdout.log 2> results/tc2/raphus.stderr.log

# run server 2 (turtur)
docker run --rm --name turtur --network proj2 -v $(pwd)/results/tc2:/app/log -e RUST_LOG=trace prj2 -n turtur -h hosts -t 2 -l log > results/tc2/turtur.stdout.log 2> results/tc2/turtur.stderr.log

# run server 3 (geopelia)
docker run --rm --name geopelia --network proj2 -v $(pwd)/results/tc2:/app/log -e RUST_LOG=trace prj2 -n geopelia -h hosts -t 2 -l log > results/tc2/geopelia.stdout.log 2> results/tc2/geopelia.stderr.log

# run server 4 (trogon)
docker run --rm --name trogon --network proj2 -v $(pwd)/results/tc2:/app/log -e RUST_LOG=trace prj2 -n trogon -h hosts -t 2 -l log > results/tc2/trogon.stdout.log 2> results/tc2/trogon.stderr.log
```

### Test Case 3

```
# run server 0 (columba)
docker run --rm --name columba --network proj2 -v $(pwd)/results/tc3:/app/log -e RUST_LOG=trace prj2 -n columba -h hosts -t 3 -l log > results/tc3/columba.stdout.log 2> results/tc3/columba.stderr.log

# run server 1 (raphus)
docker run --rm --name raphus --network proj2 -v $(pwd)/results/tc3:/app/log -e RUST_LOG=trace prj2 -n raphus -h hosts -t 3 -l log > results/tc3/raphus.stdout.log 2> results/tc3/raphus.stderr.log

# run server 2 (turtur)
docker run --rm --name turtur --network proj2 -v $(pwd)/results/tc3:/app/log -e RUST_LOG=trace prj2 -n turtur -h hosts -t 3 -l log > results/tc3/turtur.stdout.log 2> results/tc3/turtur.stderr.log

# run server 3 (geopelia)
docker run --rm --name geopelia --network proj2 -v $(pwd)/results/tc3:/app/log -e RUST_LOG=trace prj2 -n geopelia -h hosts -t 3 -l log > results/tc3/geopelia.stdout.log 2> results/tc3/geopelia.stderr.log

# run server 4 (trogon)
docker run --rm --name trogon --network proj2 -v $(pwd)/results/tc3:/app/log -e RUST_LOG=trace prj2 -n trogon -h hosts -t 3 -l log > results/tc3/trogon.stdout.log 2> results/tc3/trogon.stderr.log
```

### Test Case 4

```
# run server 0 (columba)
docker run --rm --name columba --network proj2 -v $(pwd)/results/tc4:/app/log -e RUST_LOG=trace prj2 -n columba -h hosts -t 4 -l log > results/tc4/columba.stdout.log 2> results/tc4/columba.stderr.log

# run server 1 (raphus)
docker run --rm --name raphus --network proj2 -v $(pwd)/results/tc4:/app/log -e RUST_LOG=trace prj2 -n raphus -h hosts -t 4 -l log > results/tc4/raphus.stdout.log 2> results/tc4/raphus.stderr.log

# run server 2 (turtur)
docker run --rm --name turtur --network proj2 -v $(pwd)/results/tc4:/app/log -e RUST_LOG=trace prj2 -n turtur -h hosts -t 4 -l log > results/tc4/turtur.stdout.log 2> results/tc4/turtur.stderr.log

# run server 3 (geopelia)
docker run --rm --name geopelia --network proj2 -v $(pwd)/results/tc4:/app/log -e RUST_LOG=trace prj2 -n geopelia -h hosts -t 4 -l log > results/tc4/geopelia.stdout.log 2> results/tc4/geopelia.stderr.log

# run server 4 (trogon)
docker run --rm --name trogon --network proj2 -v $(pwd)/results/tc4:/app/log -e RUST_LOG=trace prj2 -n trogon -h hosts -t 4 -l log > results/tc4/trogon.stdout.log 2> results/tc4/trogon.stderr.log
```

### Test Case 5

```
# run server 0 (columba)
docker run --rm --name columba --network proj2 -v $(pwd)/results/tc5:/app/log -e RUST_LOG=trace prj2 -n columba -h hosts -t 5 -l log > results/tc5/columba.stdout.log 2> results/tc5/columba.stderr.log

# run server 1 (raphus)
docker run --rm --name raphus --network proj2 -v $(pwd)/results/tc5:/app/log -e RUST_LOG=trace prj2 -n raphus -h hosts -t 5 -l log > results/tc5/raphus.stdout.log 2> results/tc5/raphus.stderr.log


# run server 2 (turtur)
docker run --rm --name turtur --network proj2 -v $(pwd)/results/tc5:/app/log -e RUST_LOG=trace prj2 -n turtur -h hosts -t 5 -l log > results/tc5/turtur.stdout.log 2> results/tc5/turtur.stderr.log

# run server 3 (geopelia)
docker run --rm --name geopelia --network proj2 -v $(pwd)/results/tc5:/app/log -e RUST_LOG=trace prj2 -n geopelia -h hosts -t 5 -l log > results/tc5/geopelia.stdout.log 2> results/tc5/geopelia.stderr.log

# run server 4 (trogon)
docker run --rm --name trogon --network proj2 -v $(pwd)/results/tc5:/app/log -e RUST_LOG=trace prj2 -n trogon -h hosts -t 5 -l log > results/tc5/trogon.stdout.log 2> results/tc5/trogon.stderr.log
```
