Yet Another Make Tool

see the doc on https://laurentcarrie.github.io/yamake/

but very different, some features :
- this is a crate, you build your own `make` command by creating a rust program with this crate
- rules and scanners are much easier to customize then with gnu `make`
- parallel build
- use file digest, not timestamp : avoir useless rebuild
- one log file per action, not one stdout for the whole build process
- etc
