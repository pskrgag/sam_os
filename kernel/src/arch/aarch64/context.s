.section .text

// Switches to userspace
// fn switch_to_user(*const Context);
.global switch_to_user
switch_to_user:
	// Save callee-saved register into kernel stack.
	sub	sp, sp, #0x78
	stp	x19, x20, [sp, #0]
	stp	x21, x22, [sp, #0x10]
	stp	x23, x24, [sp, #0x20]
	stp	x25, x26, [sp, #0x30]
	stp	x27, x28, [sp, #0x40]
	stp	x29, x30, [sp, #0x50]
	mov	x4, sp
	mov	x5, fp
	stp	x4, x5, [sp, #0x60]

	// Save pointer to the context, so during trap we will be able to locate it
	str	x0, [sp, #0x70]
	msr	SPSel, #1

	// Load user-space context
	ldp	x1, x2, [x0, #240]
	ldp	x3, x30, [x0, #256]

	msr	ELR_EL1, x1
	msr	SPSR_EL1, x2
	msr	SP_EL0, x3

	ldp	x2, x3, [x0, #16]
	ldp	x4, x5, [x0, #32]
	ldp	x6, x7, [x0, #48]
	ldp	x8, x9, [x0, #64]
	ldp	x10, x11, [x0, #80]
	ldp	x12, x13, [x0, #96]
	ldp	x14, x15, [x0, #112]
	ldp	x16, x17, [x0, #128]
	ldp	x18, x19, [x0, #144]
	ldp	x20, x21, [x0, #160]
	ldp	x22, x23, [x0, #176]
	ldp	x24, x25, [x0, #192]
	ldp	x26, x27, [x0, #208]
	ldp	x28, x29, [x0, #224]

	// Must be last, since x0 is the base register
	ldp	x0, x1, [x0, #0]

	// TODO: remove this flush
	tlbi	vmalle1is

	// Jump to user-space
	eret

