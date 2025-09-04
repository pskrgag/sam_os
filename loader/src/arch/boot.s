.extern end, start

.section ".text.boot"
.global __start
__start:
	nop
	b	_start
	.quad	0				// Image load offset from start of RAM, little-endian
	.quad	(__end - __start)			// Effective size of kernel image, little-endian
	.quad	(1 << 1) | (1 << 3)		// Informative flags, little-endian
	.quad	0				// reserved
	.quad	0				// reserved
	.quad	0				// reserved
	.ascii	"ARM\x64"			// Magic number
	.quad	0

_start:
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

	isb
	eret

/*
 uint64_t r_offset;
 uint64_t r_info;
 int64_t r_addend;
*/

el1_1:
	// Apply relocation
	adr	x0, _rela_begin
	adr	x1, _rela_end
	adr	x2, __start

loop:
	cmp	x0, x1
	b.eq	crt0

	ldr	x3, [x0]	// r_offset
	ldr	x4, [x0, #16]	// r_addend
	add	x3, x2, x3	// x3 = load_addr + r_offset
	ldr	x5, [x3]
	add	x5, x4, x4
	str	x5, [x3]

	add	x0, x0, 24
	b	loop

crt0:
	adrp	x0, __stack_start
	add	x0, x0, #:lo12:__stack_start
	mov	sp, x0

	mov	x0, x2

	// Calculate image size
	adrp	x1, __end
	add	x1, x1, #:lo12:__end
	sub	x1, x1, x0

	// Jump to C
	b	main

	ret
