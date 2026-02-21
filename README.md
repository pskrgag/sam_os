# SAM OS

Yet another stupid OS in Rust for no reason. Not even close to memory safe and blazingly fast.

## What that thing is all about?

I am trying to build pure capability based OS. Currently kernel does only minimal stuff: vmm, pmm, scheduling and capability maintenance.
Other stuff will live in userspace.

For now capability rights are not enforced, but will be one day.

Kernel exposes object to userspace which are referenced by capabilities:
 - Virtual memory space (VMS)
 - Virtual memory object (VMO)
 - Thread
 - Task
 - Port (IPC primitive)
 - Factory

 ... others are coming later

Userspace can create new objects via Factory object which is created by kernel on task start. After capability is obtained, userspace may do various stuff with object by invoking it via INVOKE system call.

Tasks can communicate via Ports. Port primitive is simple blob transport with support of transferring capabilities between Tasks. (NOTE: only grant is supported for capabilities for now, since revoke is kinda hard and not blazingly fast and memory safe). IPC is asynchronous and has a custom async runtime on top of it in userspace. This allows to build concurrent servers using coroutines provided by rust language.

To make life of developers (me) easier custom IDL language along with compiler generates some boilerplate code for IPC. Compiler lives under `tools/ridl` directory. Compiler generates transport code and Rust bindings using `rokio` async runtime on top of ports. Example of usage can be found in `userspace/services/roottask/src/roottask.rs`

## Supported arches
 - [x] aarch64 (qemu)

Maybe riscv64 one day. I am not messing with long, real, unreal engine 5 modes in x86 ever in my life.

## How to use it

You don't. But try smth like.

```bash
$ cargo xtask userspace/apps/console/app.toml run
```
