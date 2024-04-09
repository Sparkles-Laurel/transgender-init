# Kanit

Toy init system

## Testing

To test Kanit, the following dependencies are needed:
* `libguestfs` (`guestmount`, `guestunmount`)
* `qemu` (`qemu-system-x86_64`)
* `expect`
* `curl`
* `rust`
* `just`

Once all dependencies are installed, Kanit can be tested in a vm by running `just`.
The vm's password is set to nothing by default.

## Controller

Kanit can be controlled with `kanit`.

### Units

Units are written in TOML and can be loaded for next boot with `kanit service enable <unit> [level]`.
They must be stored at `/etc/kanit` to be found by `kanit`. Units are only ran at boot.

Units can be disabled at next boot with `kanit service disable <unit> [level]` and all enabled units can be
displayed with `kanit service list`.

### Blame

The time each unit takes to run can be viewed with `kanit blame` (or sorted with `kanit blame -s`).

### Power

The system can be powered off, rebooted, halted, and rebooted with kexec with `kanit <op>` with an
optional `-f` flag to bypass the init.

## Goal

Kanit aims to be minimal with the ability to be rolled into a single executable (multicall).

## Todo

(In no particular order)

* [x] Fix compiling with `x86_64-unknown-linux-gnu`
* [ ] Service supervision
  * [x] `kanit-supervisor`
  * [ ] Avoid spawning new process and integrate directly into `init`
  * [ ] Record logs
* [ ] Dynamically loading units
  * [ ] Move `kanit-rc/services/*` to unit files instead
  * [ ] Allow unit files to be baked into the init
  * [x] Allow unit files to be loaded at startup
* [ ] (Re)loading units
* [ ] Configuration
* [ ] Enabled units list
* [x] Concurrency
* [ ] More graceful fail over
  * [x] Emergency shell
  * [x] Allow for continuing when unit fails to start
  * [ ] Allow for panic recovery (unable to do with `panic = "abort"`)
* [x] Split `kanit-rc/src/init` to separate crate
* [ ] Consider moving `kanit-rc/init/*` to unit files
* [ ] `*dev`
  * [x] `mdev`
  * [ ] `udev`
* [ ] Testing
  * [x] Sanity boot test
  * [ ] Unit tests
* [ ] Syslog
  * [x] Busybox
  * [ ] Custom
* [ ] Logging
  * [x] Startup logger
  * [ ] Runtime logger
* [x] Shutdown

## Credit

* OpenRC/Alpine for init scripts
