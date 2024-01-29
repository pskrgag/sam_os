.macro ventry name
	b	\name
.endmacro

.macro SAFE_CTX
	sub	sp, sp, #264
	stp	x0, x1, [sp, #0]
	stp	x2, x3, [sp, #16]
	stp	x4, x5, [sp, #32]
	stp	x6, x7, [sp, #48]
	stp	x8, x9, [sp, #64]
	stp	x10, x11, [sp, #80]
	stp	x12, x13, [sp, #96]
	stp	x14, x15, [sp, #112]
	stp	x16, x17, [sp, #128]
	stp	x18, x19, [sp, #144]
	stp	x20, x21, [sp, #160]
	stp	x22, x23, [sp, #176]
	stp	x24, x25, [sp, #192]
	stp	x26, x27, [sp, #208]
	stp	x28, x29, [sp, #224]

	mrs	x0, ELR_EL1
	mrs	x1, SPSR_EL1
	mrs	x2, SP_EL0

	stp	x0, x1, [sp, #240]
	stp	x2, x30, [sp, #256]

	// Restore clobbered regs
	ldp	x0, x1, [sp, #0]
	ldr	x2, [sp, #16]
.endmacro

.macro RESTORE_CTX
	ldp	x0, x1, [sp, #240]
	ldr	x2, [sp, #256]

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
	ldr	x30, [sp, #264]

	add	sp, sp, #264
.endmacro

.section ".text"
__kern_sync64:
	SAFE_CTX

	mrs	x0, ESR_EL1
	mrs	x1, FAR_EL1
	mrs	x2, ELR_EL1
	mov	x3, sp
	bl	kern_sync64

	RESTORE_CTX
	eret

__kern_irq:
	SAFE_CTX

	bl	kern_irq

	RESTORE_CTX
	eret

__exception_bug:
	mrs	x0, ESR_EL1
	mrs	x1, FAR_EL1
	mrs	x2, ELR_EL1
	b	kern_exception_bug

__user_sync64:
	SAFE_CTX

	// x1-x5 are used for syscalls
	mrs	x6, ESR_EL1
	lsr	x6, x6, #26
	subs	x6, x6, #21		// 0b010101
	bne	sync

	mov	x0, sp
	bl	user_syscall
	b	user_ret

sync:
	mov	x0, sp
	mrs	x1, ESR_EL1
	mrs	x2, ELR_EL1
	mrs	x3, FAR_EL1
	bl	user_sync

user_ret:
	RESTORE_CTX
	eret

.section ".text"
.align 11
.global
exteption_vector:
curr_el_sp0_sync:		 // The exception handler for a synchronous
	b __exception_bug
				// exception from the current EL using SP0
.balign 0x80
curr_el_sp0_irq:         // The exception handler for an IRQ exception
	b __exception_bug
                         // from the current EL using SP0.
.balign 0x80
curr_el_sp0_fiq:         // The exception handler for an FIQ exception
	b __exception_bug
                         // from the current EL using SP0.
.balign 0x80
curr_el_sp0_serror:      // The exception handler for a System Error
	b __exception_bug
                         // exception from the current EL using SP0.
.balign 0x80
curr_el_spx_sync:        // The exception handler for a synchrous
                         // exception from the current EL using the
                         // current SP.
	ventry __kern_sync64
.balign 0x80
curr_el_spx_irq:         // The exception handler for an IRQ exception from
                         // the current EL using the current SP.
	ventry __kern_irq
	b .

.balign 0x80
curr_el_spx_fiq:         // The exception handler for an FIQ from
                         // the current EL using the current SP.
	b __exception_bug

.balign 0x80
curr_el_spx_serror:      // The exception handler for a System Error
                         // exception from the current EL using the
                         // current SP.
	b __exception_bug
 .balign 0x80
lower_el_aarch64_sync:   // The exception handler for a synchronous
                         // exception from a lower EL (AArch64).
	ventry __user_sync64
.balign 0x80
lower_el_aarch64_irq:    // The exception handler for an IRQ from a lower EL
                         // (AArch64).
	ventry __kern_irq

.balign 0x80
lower_el_aarch64_fiq:    // The exception handler for an FIQ from a lower EL
                         // (AArch64).
	ventry __kern_sync64

.balign 0x80
lower_el_aarch64_serror: // The exception handler for a System Error
                         // exception from a lower EL(AArch64).
	ventry __exception_bug

.balign 0x80
lower_el_aarch32_sync:   // The exception handler for a synchronous
                         // exception from a lower EL(AArch32).
	ventry __kern_sync64
.balign 0x80
lower_el_aarch32_irq:    // The exception handler for an IRQ exception
                         // from a lower EL (AArch32).
	ventry __kern_sync64
.balign 0x80
lower_el_aarch32_fiq:    // The exception handler for an FIQ exception from
                         // a lower EL (AArch32).
	ventry __kern_sync64
.balign 0x80
lower_el_aarch32_serror: // The exception handler for a System Error
                         // exception from a lower EL(AArch32).
	ventry __exception_bug

.previous
