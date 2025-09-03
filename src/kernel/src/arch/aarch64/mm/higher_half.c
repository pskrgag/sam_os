typedef unsigned long uint64_t;
typedef unsigned long uintptr_t;

typedef unsigned int uint32_t;

#define PAGE_SIZE (1 << 12)
#define UL(x) ((unsigned long)(x))

extern char __start[];
extern char __end[];

extern void __attribute__((noreturn)) start_kernel(unsigned long load_addr);
extern void __attribute__((noreturn)) cpu_reset(void);

typedef uint64_t tte_t;

#if defined(CONFIG_BOARD_QEMU)
#define UART_BASE 0x09000000
#elif defined(CONFIG_BOARD_ORPIPC2)
#define UART_BASE 0x01C28000
#else
#error "Misconfiguration"
#endif

static tte_t lvl1[512] __attribute__((aligned(4096)));
static tte_t lvl2_image_1to1[512] __attribute__((aligned(4096)));
static tte_t lvl2_image_higher[512] __attribute__((aligned(4096)));
static tte_t lvl2_mmio[512] __attribute__((aligned(4096)));

static uint64_t l1_linear_offset(void *p)
{
	uint64_t va = (uint64_t)p;

	return (va >> 30) & 511;
}

static uint64_t l2_linear_offset(void *p)
{
	uint64_t va = (uint64_t)p;

	return (va >> 21) & 511;
}

static inline void tmp_printf(const char *ptr)
{
#if defined(CONFIG_BOARD_ORPIPC2)
	while ((*((volatile uint32_t *)UART_BASE + 5) & 0x40) == 0) {
	}
#endif

	for (; *ptr; ptr++)
		*(volatile char *)(uintptr_t)UART_BASE = *ptr;
}

static inline void mmio_1_v_1(void)
{
	tte_t device_lvl1 = UL(&lvl2_mmio) | 0b11;
	tte_t *lvl2;

	if (lvl1[l1_linear_offset((void *) UART_BASE)] == 0) {
		lvl1[l1_linear_offset((void *) UART_BASE)] = device_lvl1;
		lvl2 = lvl2_mmio;
	} else {
		lvl2 = lvl2_image_1to1;
	}

	tte_t device_lvl2 = UART_BASE | (1 << 10) | (1 << 2) | 0b01;
	lvl2[l2_linear_offset((void *)UART_BASE)] = device_lvl2;
}

__attribute__((noinline)) static void print_int(unsigned long num)
{
	char out[100] = {};

	long digits = 1;
	for (long nn = num; nn /= 10; digits++)
		;
	for (int i = digits - 1; i >= 0; i--) {
		out[i] = '0' + (num % 10);
		num /= 10;
	}

	tmp_printf(out);
}

static inline void map_image(unsigned long load_addr, unsigned long image_size)
{
	tte_t lvl2_1to1 = UL(&lvl2_image_1to1) | 0b11;
	tte_t lvl2_higher = UL(&lvl2_image_higher) | 0b11;
	unsigned long start_phys = load_addr;
	unsigned long start_virt = 0xffffffa000000000;

	lvl1[l1_linear_offset((void *)start_phys)] = lvl2_1to1;
	lvl1[l1_linear_offset((void *)start_virt)] = lvl2_higher;

	for (; image_size; image_size -= (2 << 20), start_phys += (2 << 20),
			   start_virt += (2 << 20)) {
		tte_t tte = start_phys | (1 << 10) | 0b01;

		lvl2_image_1to1[l2_linear_offset((void *)start_phys)] = tte;
		lvl2_image_higher[l2_linear_offset((void *)start_virt)] = tte;
	}
}

__attribute__((section(".text.boot"))) void map(unsigned long load_addr,
						unsigned long image_size)
{
	uint64_t tcr = (25UL << 16) | 25 | (2UL << 30) | (3UL << 26) |
		       (3UL << 24) | (3UL << 8) | (3UL << 10);
	uint64_t mair = (0b00000000 << 8) | 0b01110111;
	uint64_t ttbr_el1 = ((uint64_t)(void *)&lvl1);
	uint64_t sctrl;
	void (*rust_start_higher_half)(unsigned long) = (void *)(&start_kernel);

	map_image(load_addr, image_size);
	mmio_1_v_1();

	asm volatile("msr TCR_EL1, %0" ::"r"(tcr));
	asm volatile("msr MAIR_EL1, %0" ::"r"(mair));

	asm volatile("msr TTBR0_EL1, %0" ::"r"(ttbr_el1));
	asm volatile("msr TTBR1_EL1, %0" ::"r"(ttbr_el1));
	asm volatile("mrs %0, SCTLR_EL1" : "=r"(sctrl));

	sctrl = (1 << 0) | (1 << 2);

	tmp_printf("Hello");
	asm volatile("tlbi vmalle1\n"
		     "isb\n"
		     "dsb ishst\n"
		     "msr SCTLR_EL1, %0\n"
		     "dsb ishst\n"
		     "isb\n" ::"r"(sctrl));

	tmp_printf("Set up minimal page_table... Jumping to Rust code\n\r");

	asm volatile("br	%0\nb	." ::"r"(rust_start_higher_half));

	/* LLD is smart enough to optimize out all rust code if there won't be any references */
	start_kernel((unsigned long)__start);
}

void __attribute__((section(".text.boot"))) reset(void)
{
	uint64_t tcr = (25UL << 16) | 25 | (2UL << 30) | (3UL << 26) |
		       (3UL << 24) | (3UL << 8) | (3UL << 10);
	uint64_t mair = (0b00000000 << 8) | 0b01110111;
	uint64_t sctrl;
	void (*rust_reset)(void) = (void *)(&cpu_reset);
	uint64_t ttbr_el1 = ((uint64_t)(void *)&lvl1);

	asm volatile("msr TTBR0_EL1, %0" ::"r"(ttbr_el1));
	asm volatile("msr TCR_EL1, %0" ::"r"(tcr));
	asm volatile("msr MAIR_EL1, %0" ::"r"(mair));
	asm volatile("tlbi    vmalle1is");
	asm volatile("mrs %0, SCTLR_EL1" : "=r"(sctrl));

	sctrl = ((1 << 0) | (1 << 2) | (1 << 12));

	asm volatile("msr SCTLR_EL1, %0" ::"r"(sctrl));
	asm volatile("br	%0" ::"r"(rust_reset));

	cpu_reset();
}
