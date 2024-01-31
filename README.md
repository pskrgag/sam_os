# SAM OS

Yet another stupid OS in Rust for no reason. Not even close to memory safe and blazingly fast.

## What that thing is all about?

I am trying to build pure capability based OS. Currently kernel does only minimal stuff: vmm, pmm, scheduling and capability maintenance. Other stuff will live in userspace.
Kernel supports only 3 syscalls (one is for debugging): write to console, invoke capability and yield thread from the CPU. That's all.

Kernel exports object to userspace which are referenced by capabilities:
 - Virtual memory space (VMS)
 - Virtual memory object (VMO)
 - Thread
 - Task
 - Port (IPC primitive)
 - Factory

 ... others comming later

Userspace can create new objects via Factory object which is created by kernel on task start. After capability is obtained, userspace may do various stuff with object by invoking it via INVOKE system call.

Tasks can communicate via Ports. Port primitive is simple blob transport with support of transfering capabilities between Tasks. (NOTE: only grant is supported for capabilities for now, since revoke is kinda hard and not blazingly fast and memory safe). IPC is synchronous, since it makes life a lot easier.

To make life of developers (me) easier custom IDL language along with compiler generates some boilerplate code for transfer logic. Compiler lives under `ridl` directory and `.idl` files live under `userspace/interfaces` directory. 

## Supported arches
 - [x] aarch64 (qemu only for now)

Maybe riscv64 one day. I am not messing with long, real, unreal engine 5 modes in x86 ever in my life.   

## How to use it

You don't

## How to run it

You will need nighly rust compiler with  aarch64-unknown-none-softfloat support. Hit 
```
$ make console
$ make qemu
```
and maybe you will see simple console. If not -- idk; works on my machine. 

## TODO
List of things I am planning to do in near future (maybe)
 - [x] Custom build system, which will support dependencies from interfaces (i don't like how it looks, but anyway)
 - [ ] Sane VMM
 - [ ] SMP
 - [ ] IRQ object (to support more drivers)
 - [ ] Port some FS
 - [ ] Clean up all TODOs across code base
