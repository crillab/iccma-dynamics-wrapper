# ICCMA-DYNAMICS-WRAPPER

The solver wrapper for dynamic tracks in ICCMA'21 competition.

## Changes for dynamic tracks at ICCMA'21

ICCMA'19 introduced dynamics tracks, in which the solvers were intended to solve a problem for a dynamic argumentation framework, i.e. frameworks for which new attacks are added/removed over the time.

Dynamic frameworks were defined using two files:

* an _initial framework_, encoded the same way frameworks were encoded for static tracks;
* a _dynamics file_, where an attack addition/deletion was given at each line.

Given a dynamic framework with `n` attacks in the dynamics file, solvers were intended to solve `n+1` problems for each query:

1. solver the initial problem,
2. solve the initial problem altered by the first line in the dynamics file,
3. solve the initial problem altered by the first and the second line in the dynamics file,
4. and so on...

Dynamics tracks were designed to reflect situations in which new arguments are added (e.g. in a debate), for which we have no idea which arguments and attacks will be added or removed.
However, having a full list of modifications before any calculation is not realistic in application of dynamic argumentation. Moreover, some solvers could take advantage of having the full list of modifications in their algorithm.
This program prevents this behavior.

## How does it work

Just like a regular dynamics solver of the ICCMA'19 competition, the `ICCMA-DYNAMICS-WRAPPER` (IDW in the following) takes as input a (dynamic) problem, an initial argumentation framework, a dynamics file, the encoding used in these two files, and an argument in case the query is `DC` or `DS`. It takes as an additional parameter the solver under consideration.

At startup, IDW executes the underlying solver with the arguments corresponding to the problem, the initial AF, the encoding, and the argument if needed; it then waits for the solver answer. Then, for each line in the dynamics file, IDW writes it in the standard input of the solver and waits for a new response from the solver. After the `n+1` answers were read, IDW writes an empty line on the solver's standard input to inform it no more computation is required.

## Building and executing IDW

First, you need a recent version of the Rust toolchain (including the `cargo` tool); go to [rust-lang.org](https://www.rust-lang.org/tools/install) to install it if needed. In case you already installed it, you may need to update it with the command `rustup update`.

From inside the source directory of IDW, run `cargo build` to compile the program. Now, you can execute it with the following command.

```
cargo run -- wrap -p PROBLEM -f AF_FILE -m DYN_FILE -z INSTANCE_FORMAT [-a ARG] -s SOLVER
```

Note that the command line arguments are the same the ICCMA'19 dynamic solvers, except that `-fo` is replaced by `-z` and that `-s` must provide the underlying solver. Here is an example of an execution with a dynamics file involving two attacks, for which the solvers answers `[a, b]`, `[a]`, and finally `[]`.

```
me@my-machine:~/iccma21-dynamics-wrapper$ cargo run -- wrap -p SE-CO-D -f ./instance.apx -z apx -m ./instance-mod.apxm -s $PWD/utils/fake-solver.sh 
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
     Running `target/debug/iccma21-dynamics-wrapper wrap -p SE-CO-D -f ./instance.apx -z apx -m ./instance-mod.apxm -s /home/lonca/projects/iccma21/iccma21-dynamics-wrapper/utils/fake-solver.sh`
[INFO ] [2020-11-26 11:54:24] iccma21-dynamics-wrapper 0.1.0
[a, b]
[a]
[]
[INFO ] [2020-11-26 11:55:25] exiting successfully after 60.857233044s

```

## License

The _iccma-dynamics-wrapper_ is developed at CRIL (Centre de Recherche en Informatique de Lens).
It is made available under the terms of the GNU GPLv3 license.