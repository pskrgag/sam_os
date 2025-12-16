.section .text

.macro safe_context reason, user
.if \user
	// Save x5
	str	x5, [sp, #-8]!

	// Load pointer to the context
	ldr	x5, [sp, #0x78]

	// TODO: pass tmp registers as part of argument list
	__safe_context \reason, x5

	// Fixup x5
	mov	x6, x5
	ldr	x5, [sp], 8
	str	x5, [x6, #40]
.else
	sub	sp, sp, #280
	__safe_context \reason, sp
.endif
.endm

.macro __safe_context reason, reg
	stp	x0, x1, [\reg, #0]
	stp	x2, x3, [\reg, #16]
	stp	x4, x5, [\reg, #32]
	stp	x6, x7, [\reg, #48]
	stp	x8, x9, [\reg, #64]
	stp	x10, x11, [\reg, #80]
	stp	x12, x13, [\reg, #96]
	stp	x14, x15, [\reg, #112]
	stp	x16, x17, [\reg, #128]
	stp	x18, x19, [\reg, #144]
	stp	x20, x21, [\reg, #160]
	stp	x22, x23, [\reg, #176]
	stp	x24, x25, [\reg, #192]
	stp	x26, x27, [\reg, #208]
	stp	x28, x29, [\reg, #224]

	mrs	x0, ELR_EL1
	mrs	x1, SPSR_EL1
	mrs	x2, SP_EL0
	mov	x3, \reason

	stp	x0, x1, [\reg, #240]
	stp	x2, x30, [\reg, #256]
	str	x3, [\reg, #272]
.endm

.macro restore_context_kernel
	ldp	x0, x1, [sp, #240]
	ldp	x2, x30, [sp, #256]

	msr	ELR_EL1, x0
	msr	SPSR_EL1, x1
	msr	SP_EL0, x2

	ldp	x0, x1, [sp, #0]
	ldp	x2, x3, [sp, #16]
	ldp	x4, x5, [sp, #32]
	ldp	x6, x7, [sp, #48]
	ldp	x8, x9, [sp, #64]
	ldp	x10, x11, [sp, #80]
	ldp	x12, x13, [sp, #96]
	ldp	x14, x15, [sp, #112]
	ldp	x16, x17, [sp, #128]
	ldp	x18, x19, [sp, #144]
	ldp	x20, x21, [sp, #160]
	ldp	x22, x23, [sp, #176]
	ldp	x24, x25, [sp, #192]
	ldp	x26, x27, [sp, #208]
	ldp	x28, x29, [sp, #224]

	add	sp, sp, #280
.endm

// `reason` is RawTrapReason enum
.macro ventry reason user
	safe_context \reason, \user
	b	generic_handler
.endm

generic_handler:
	// Check M[0:3] bits. 0 means EL0
	mrs	x0, SPSR_EL1
	tst	x0, #0x7
	beq	user_trap

	// 1st argument is pointer to ExceptionCtx. Exception reason is already saved in it.
	mov	x0, sp
	bl	trap_handler

	restore_context_kernel
	eret

user_trap:
	// Kernel context is saved on the kernel stack. Restore it and jump to handler loop
	ldp	x19, x20, [sp, 0]
	ldp	x21, x22, [sp, 0x10]
	ldp	x23, x24, [sp, 0x20]
	ldp	x25, x26, [sp, 0x30]
	ldp	x27, x28, [sp, 0x40]
	ldp	x29, x30, [sp, 0x50]
	ldp	x4, x5,   [sp, 0x60]
	mov	sp, x4
	mov	fp, x5
	add	sp, sp, 0x78

	// return to the previous kernel frame
	ret

.align 11
.global exception_vector
exception_vector:
curr_el_sp0_sync:		// The exception handler for a synchronous
	ventry	2, 0		// exception from the current EL using SP0

.balign 0x80
curr_el_sp0_irq:		// The exception handler for an IRQ exception
	ventry	0, 0		// from the current EL using SP0.

.balign 0x80
curr_el_sp0_fiq:		// The exception handler for an FIQ exception
	ventry	1, 0		// from the current EL using SP0.

.balign 0x80
curr_el_sp0_serror:		// The exception handler for a System Error
	ventry	3, 0		// exception from the current EL using SP0.

.balign 0x80
curr_el_spx_sync:		// The exception handler for a synchrous
				// exception from the current EL using the
				// current SP.
	ventry	2, 0

.balign 0x80
curr_el_spx_irq:		// The exception handler for an IRQ exception from
	ventry	0, 0		// the current EL using the current SP.

.balign 0x80
curr_el_spx_fiq: 		// The exception handler for an FIQ from
				// the current EL using the current SP.
	ventry	1, 0

.balign 0x80
curr_el_spx_serror:		// The exception handler for a System Error
				// exception from the current EL using the
				// current SP.
	ventry	3, 0

.balign 0x80
lower_el_aarch64_sync:		// The exception handler for a synchronous
				// exception from a lower EL (AArch64).
	ventry	2, 1

.balign 0x80
lower_el_aarch64_irq:		// The exception handler for an IRQ from a lower EL
				// (AArch64).
	ventry	0, 1

.balign 0x80
lower_el_aarch64_fiq:		// The exception handler for an FIQ from a lower EL
				// (AArch64).
	ventry	1, 1

.balign 0x80
lower_el_aarch64_serror:	// The exception handler for a System Error
				// exception from a lower EL(AArch64).
	ventry	3, 1

.balign 0x80
lower_el_aarch32_sync:		// The exception handler for a synchronous
				// exception from a lower EL(AArch32).
	ventry	2, 1

.balign 0x80
lower_el_aarch32_irq:		// The exception handler for an IRQ exception
				// from a lower EL (AArch32).
	ventry 0, 1

.balign 0x80
lower_el_aarch32_fiq:		// The exception handler for an FIQ exception from
				// a lower EL (AArch32).
	ventry 1, 1

.balign 0x80
lower_el_aarch32_serror:	// The exception handler for a System Error
				// exception from a lower EL(AArch32).
	ventry 3, 1
