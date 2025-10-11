# SAM OS

Yet another stupid OS in Rust for no reason. Not even close to memory safe and blazingly fast.

## What that thing is all about?

I am trying to build pure capability based OS. Currently kernel does only minimal stuff: vmm, pmm, scheduling and capability maintenance.
Other stuff will live in userspace.

For now capability rights are not enforced, but will one.

Kernel exports object to userspace which are referenced by capabilities:
 - Virtual memory space (VMS)
 - Virtual memory object (VMO)
 - Thread
 - Task
 - Port (IPC primitive)
 - Factory

 ... others are coming later

Userspace can create new objects via Factory object which is created by kernel on task start. After capability is obtained, userspace may do various stuff with object by invoking it via INVOKE system call.

Tasks can communicate via Ports. Port primitive is simple blob transport with support of transferring capabilities between Tasks. (NOTE: only grant is supported for capabilities for now, since revoke is kinda hard and not blazingly fast and memory safe). IPC is synchronous, since it makes life a lot easier.

To make life of developers (me) easier custom IDL language along with compiler generates some boilerplate code for transfer logic. Compiler lives under `tools/ridl` directory and

## Supported arches
 - [x] aarch64 (qemu)

Maybe riscv64 one day. I am not messing with long, real, unreal engine 5 modes in x86 ever in my life.

## How to use it

You don't
