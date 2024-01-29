
.macro uld1	to, from, scratch, fixup
i\@:	ldrb	\scratch, [\from]
	strb	\scratch, [\to]
.pushsection ".fixup"
	.quad	i\@, fixup
.popsection
.endm

.macro ust1	to, from, scratch, fixup
	ldrb	\scratch, [\from]
i\@:	strb	\scratch, [\to]
.pushsection ".fixup"
	.quad	i\@, fixup
.popsection
.endm

// TODO for simplicity just use byte-by-byte copy, but in future
// it would be nice to have loops over 16, 8 and 4 bytes
// x0 -- source VA
// x1 -- size of the buffer
// x2 -- destintaion VA
.text
.globl arch_copy_from_user
arch_copy_from_user:
loop:
	// Check if size is zero
	cmp	x1, xzr
	beq	done

	uld1	x2, x0, w4, fixup

	add	x0, x0, #1
	add	x2, x2, #1
	sub	x1, x1, #1
	b	loop

done:
	mov	x0, xzr
	ret

fixup:
	mov	x0, #-1
	ret

// x0 -- source VA
// x1 -- size of the buffer
// x2 -- destintaion VA
.globl arch_copy_to_user
arch_copy_to_user:
l:
	// Check if size is zero
	cmp	x1, xzr
	beq	d

	ust1	x2, x0, w4, fixup

	add	x0, x0, #1
	add	x2, x2, #1
	sub	x1, x1, #1
	b	l

d:
	mov	x0, xzr
	ret

