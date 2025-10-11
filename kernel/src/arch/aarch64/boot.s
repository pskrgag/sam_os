.extern end, start

.section ".text.boot"
.global __start
__start:
	// Setup stack
	ldr	x1, =__STACK_START
	mov	sp, x1

	// Jump to Rust
	bl	start_kernel
	b	.

// .global __reset
// __reset:
// 	mrs	x0, MPIDR_EL1
// 	and	x8, x0, #0xffffffffff
// 	and	x0, x8, #0xffffffff00ffffff
// 
// 	// Load stack array
// 	adrp	x1, IDLE_THREAD_STACK
// 	add	x1, x1, #:lo12:IDLE_THREAD_STACK
// 
// 	// x0 -- CPUID
// 	// x0 << 3 == x0 * 8 == x0 * 2^3
// 	ldr     x1, [x1, x0, lsl #3]
// 
// 	mov	sp, x1
// 
// 	// For some reason my compiler does not want to genetate PIC code for static from Rust
// 	// I guess I was doing smth wrong
// 	// So load TTBR1 here
// 	adrp	x0, PAGE_TABLE_BASE
// 	add	x0, x0, #:lo12:PAGE_TABLE_BASE
// 	ldr	x0, [x0]
// 	msr	TTBR1_EL1, x0
// 
// 	b	reset
// 
// // Park CPU -- should not reach here
// 	b	.
