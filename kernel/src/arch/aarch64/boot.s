.section ".text.boot"

.global __start
__start:
	adrp	x0, __STACK_START
	add	x0, x0, #:lo12:__STACK_START
	mov	sp, x0

	b	map
