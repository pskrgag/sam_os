typedef unsigned long uint64_t;
typedef unsigned long uintptr_t;

typedef unsigned int uint32_t;

#define PAGE_SIZE	(1 << 12)
#define UL(x)		((unsigned long) (x))

extern uint64_t start;
extern uint64_t load_addr;
extern uint64_t kernel_virtual_base;

extern uint64_t mmio_start;
extern uint64_t mmio_end;
extern uint64_t mmio_base;

extern void __attribute__((noreturn)) start_kernel(void);
extern void __attribute__((noreturn)) cpu_reset(void);

#define ROUND_UP(x, y)	((((x) - 1) | ((y) - 1)) + 1)

typedef uint64_t tte_t;

#if defined(CONFIG_BOARD_QEMU)
#define UART_BASE	0x09000000
#elif defined(CONFIG_BOARD_ORPIPC2)
#define UART_BASE	0x01C28000
#else
# error "Misconfiguration"
#endif

static tte_t lvl1[512] __attribute__((aligned(4096)));
static tte_t lvl2[512] __attribute__((aligned(4096)));

static uint64_t l1_linear_offset(void *p)
{
	uint64_t va = (uint64_t) p;

	return (va >> 30) & (512 - 1);
}

static uint64_t l2_linear_offset(void *p)
{
	uint64_t va = (uint64_t) p;

	return (va >> 21) & (512 - 1);
}

static inline void tmp_printf(const char *ptr)
{
#if defined(CONFIG_BOARD_ORPIPC2)
	while ((*((volatile uint32_t *) UART_BASE + 5) & 0x40) == 0) {}
#endif

	for (; *ptr; ptr++)
		*(volatile char *) (uintptr_t)UART_BASE = *ptr;

}


static inline void mmio_1_v_1(void)
{
	tte_t device_lvl1 = UL(&lvl2) | 0b11;
	unsigned long mmio_size = ROUND_UP(UART_BASE - UL(&mmio_base) + 1, (1 << 21));
	void *mmio_addr = &mmio_base;

	lvl1[l1_linear_offset(mmio_addr)] = device_lvl1;

	for (; mmio_size; mmio_size -= (2 << 20), mmio_addr += (2 << 20)) {
		tte_t device_lvl2 = UL(mmio_addr) | (1 << 10) | (1 << 2) | 0b01;

		lvl2[l2_linear_offset(mmio_addr)] = device_lvl2;
	}
}

__attribute__((section(".text.boot"))) void map(void)
{
	tte_t _1_v_1_1gb = UL(&load_addr) | (1 << 10) |  0b01;
	uint64_t tcr = (25UL << 16) | 25 | (2UL << 30) | (3UL << 26) | (3UL << 24) | (3UL << 8) | (3UL << 10);
	uint64_t mair = (0b00000000 << 8) | 0b01110111;
	uint64_t ttbr_el1 = ((uint64_t) (void *) &lvl1);
	uint64_t sctrl;
	void (*rust_start_higher_half)(void) = (void *) (&start_kernel);

	lvl1[l1_linear_offset(&load_addr)] = _1_v_1_1gb;
	lvl1[l1_linear_offset(&kernel_virtual_base)] = _1_v_1_1gb;

	mmio_1_v_1();

	asm volatile ("msr TCR_EL1, %0"::"r"(tcr));
	asm volatile ("msr MAIR_EL1, %0"::"r"(mair));

	asm volatile ("msr TTBR0_EL1, %0"::"r"(ttbr_el1):"memory");
	asm volatile ("msr TTBR1_EL1, %0"::"r"(ttbr_el1));
	asm volatile ("tlbi    vmalle1");

	asm volatile ("mrs %0, SCTLR_EL1": "=r"(sctrl));

	sctrl = ((1 << 0) | (1 << 2) | (1 << 12));

	asm volatile ("isb");
	asm volatile ("dsb ishst");
	asm volatile ("msr SCTLR_EL1, %0"::"r"(sctrl));
	asm volatile ("dsb ishst");
	asm volatile ("isb");

	tmp_printf("Set up minimal page_table... Jumping to Rust code\n\r");

	asm volatile ("br	%0"::"r"(rust_start_higher_half));
	asm ("b		."::: "memory");

	/* Rust is smart enough to optimze out all rust code if there won't be any references */
	start_kernel();
}

void __attribute__((section(".text.boot"))) reset(void)
{
	uint64_t tcr = (25UL << 16) | 25 | (2UL << 30) | (3UL << 26) | (3UL << 24) | (3UL << 8) | (3UL << 10);
	uint64_t mair = (0b00000000 << 8) | 0b01110111;
	uint64_t sctrl;
	void (*rust_reset)(void) = (void *) (&cpu_reset);
	uint64_t ttbr_el1 = ((uint64_t) (void *) &lvl1);

	asm volatile ("msr TTBR0_EL1, %0"::"r"(ttbr_el1));
	asm volatile ("msr TCR_EL1, %0"::"r"(tcr));
	asm volatile ("msr MAIR_EL1, %0"::"r"(mair));
	asm volatile ("tlbi    vmalle1is");
	asm volatile ("mrs %0, SCTLR_EL1": "=r"(sctrl));

	sctrl = ((1 << 0) | (1 << 2) | (1 << 12));

	asm volatile ("msr SCTLR_EL1, %0"::"r"(sctrl));
	asm volatile ("br	%0"::"r"(rust_reset));

	cpu_reset();
}
