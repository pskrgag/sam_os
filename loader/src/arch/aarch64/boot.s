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
	mov	x10, x0

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

el1_1:
	// Apply relocation
	adr	x0, _rela_begin
	adr	x1, _rela_end
	adr	x2, __start

loop:
	cmp	x0, x1
	b.eq	crt0

	ldp	x3, x4, [x0], #24
	ldr	x5, [x0, #-8]

	// x3 - r_offset
	// x4 - r_info
	// x5 - r_addent

	add	x5, x5, x2	// x5 = r_offset + __start
	str	x5, [x3, x2]
	b	loop

crt0:
	adrp	x0, __end
	add	x0, x0, #:lo12:__end
	mov	sp, x0

	mov	x0, x10

	// Jump to Rust
	b	main

	ret
