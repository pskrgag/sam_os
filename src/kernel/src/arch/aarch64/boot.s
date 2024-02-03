.section ".text.boot"

.global __start
__start:
	bl	hello
	mrs	x0, CurrentEl
	cmp	x0, #(1 << 2)
	b.eq	el1_1

	/* Only EL2 supported, too tired for el3 boilerplate. why tf you even
	 * want to boot OS from el3?. */
	msr	SCTLR_EL1, xzr

	mov	x0, xzr
	orr	x0, x0, #(1 << 31)
	orr	x0, x0, #(1 << 29)
	msr	HCR_EL2, x0

	mov	x0, #(0b00101 | (1 << 7) | (1 << 6) | (1 << 9) | (1 << 8))
	msr	SPSR_EL2, x0

	adr	x1, el1_1
	msr	ELR_EL2, x1

	mrs	x1, VBAR_EL2
	msr	VBAR_EL1, x1

	bl	hello
	isb
	eret
el1_1:
	bl	hello
	adrp	x0, __STACK_START
	add	x0, x0, #:lo12:__STACK_START
	mov	sp, x0

	b	map

.global __reset
__reset:
	mrs	x0, MPIDR_EL1
	and     x8, x0, #0xffffffffff
	and     x0, x8, #0xffffffff00ffffff

	// Load stack array
	adrp	x1, IDLE_THREAD_STACK
	add	x1, x1, #:lo12:IDLE_THREAD_STACK

	// x0 -- CPUID
	// x0 << 3 == x0 * 8 == x0 * 2^3
	ldr     x1, [x1, x0, lsl #3]

	mov	sp, x1

	// For some reason my compiler does not want to genetate PIC code for static from Rust
	// I guess I was doing smth wrong
	// So load TTBR1 here
	adrp	x0, PAGE_TABLE_BASE
	add	x0, x0, #:lo12:PAGE_TABLE_BASE
	ldr	x0, [x0]
	msr	TTBR1_EL1, x0

	b	reset

// Park CPU -- should not reach here
	b	.
