# are we sdd yet?

This repo is a work-in-progress benchmarking suite comparing [rsdd](https://github.com/neuppl/rsdd) to other decision diagram libraries. The goal is to provide a simple, consistent, and **reproducible** set of tools to produce results about the tool.

Currently, this tool only supports head-to-head benchmarks against the [`sdd` package](http://reasoning.cs.ucla.edu/sdd/) and the [`cnf2obdd` tool](http://www.sd.is.uec.ac.jp/toda/code/cnf2obdd.html).

Some future plans:

- autogenerate a website to view statistics
- integrate CUDD (major blockers: lack of out-of-the-box CLI, BLIF format)
- integrate Sylvan
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

Some helpful arguments:

- `-o` allows you to specify an output file to dump benchmark runs to, which is great for visualization
  - ex, in the container, run `are_we_sdd_yet -f fixtures/cnf/s208.1.scan.cnf -o output.json && cat output.json`
- `-m` allows you to pick the compilation mode. Right now, the only reasonable options are:
  - `-m right` (the default): right linear SDD
  - `-m best`: the 'best' static option; this is right linear for the sdd library (seems wrong), and the DTree heuristic for rsdd
  - `-m best-bdd`/`-m bdd-best`: the 'best' static option, limited to BDDs only; this is a more fair comparison for cnf2obdd
  - `-m left`: left linear SDD. this is *extraordinarily* inefficient!


## Attribution and Licensing

This repository is MIT licensed.

It bundles the [SDD Package](http://reasoning.cs.ucla.edu/sdd/) to properly benchmark against it; the Apache License for the project is included in the relevant directories.

It bundles the [CNF2OBDD tool](http://www.sd.is.uec.ac.jp/toda/code/cnf2obdd.html) to properly benchmark against it; the (inherited) MIT License for the project is included in the relevant directories.
