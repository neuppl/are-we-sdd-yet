# are we sdd yet?

This repo is a work-in-progress benchmarking suite comparing [rsdd](https://github.com/neuppl/rsdd) to other decision diagram libraries. The goal is to provide a simple, consistent, and **reproducible** set of tools to produce results about the tool.

Currently, this tool only supports head-to-head benchmarks against `sdd`.

Some future plans:

- autogenerate a website to view statistics
- integrate CUDD (major blockers: lack of out-of-the-box CLI, BLIF format)
- benchmark other tasks:
  - marginal map
  - reachability
  - ...
- better Docker engineering

And, many other things :)

## Usage

The core engine for this suite uses [Docker](https://www.docker.com/); please install it before procedding.

Then, you can build and interact with the `awsy` container, which is the top-level entrypoint for the suite.

```
$ docker compose build awsy
```

This will build the dependent images (for now, `rsdd` and `sdd`). You can now run the `are-we-sdd-yet` benchmarking tool through the container. It takes in a variadic number of `-f`s, each of which are a CNF file to benchmark on.

```
$ docker compose run awsy are_we_sdd_yet -f fixtures/cnf/cm152a.cnf -f fixtures/cnf/s208.1.scan.cnf -f fixtures/cnf/x2.cnf
[+] Running 2/2
 ✔ Container are-we-sdd-yet-rsdd-1  Created                                                                                                    0.0s
 ✔ Container are-we-sdd-yet-sdd-1   Recreated                                                                                                  0.1s
[+] Running 2/2
 ✔ Container are-we-sdd-yet-sdd-1   Started                                                                                                    0.4s
 ✔ Container are-we-sdd-yet-rsdd-1  Started                                                                                                    0.4s
Compiling fixtures/cnf/cm152a.cnf with vtree strategy right
0.27x speedup (rsdd: 0.004390s, sdd: 0.001187s)
Compiling fixtures/cnf/s208.1.scan.cnf with vtree strategy right
2.27x speedup (rsdd: 0.929565s, sdd: 2.113326s)
Compiling fixtures/cnf/x2.cnf with vtree strategy right
1.28x speedup (rsdd: 0.007818s, sdd: 0.010031s)
```

## Attribution and Licensing

This repository is MIT licensed. It also bundles the [SDD Package](http://reasoning.cs.ucla.edu/sdd/) to properly benchmark against it; the Apache License for the project is included in the relevant directories.
