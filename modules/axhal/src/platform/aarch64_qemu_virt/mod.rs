pub mod mem;

use crate::mem::phys_to_virt;
use crate::misc::terminate;
use core::arch::asm;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::aarch64_common::gic::*;
}

pub mod console {
    pub use crate::platform::aarch64_common::pl011::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
}

pub mod misc {
    pub use crate::platform::aarch64_common::psci::system_off as terminate;
}

unsafe extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    axcpu::init::init_trap();
    crate::cpu::init_primary(cpu_id);
    super::aarch64_common::pl011::init_early();
    super::aarch64_common::generic_timer::init_early();
    rust_main(cpu_id, dtb);
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    axcpu::init::init_trap();
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_primary();
    super::aarch64_common::generic_timer::init_percpu();
    super::aarch64_common::pl011::init();

    init_gpio_interrupt();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}
pub fn init_gpio_interrupt() {
    let gpio_phys_addr = pa!(axconfig::devices::GPIO_PADDR);
    let gpio_virt_addr = phys_to_virt(gpio_phys_addr).as_mut_ptr();

    // GPIO 中断使能寄存器偏移地址
    const GPIO_INTERRUPT_ENABLE_OFFSET: usize = 0x410;
    // 使能 GPIO 第3号引脚中断的位掩码
    const GPIO_PIN3_ENABLE_BIT: u8 = 1 << 3;

    info!("Enabling GPIO pin 3 interrupt");

    unsafe {
        let interrupt_enable_reg = gpio_virt_addr.add(GPIO_INTERRUPT_ENABLE_OFFSET) as *mut u8;
        core::ptr::write_volatile(interrupt_enable_reg, GPIO_PIN3_ENABLE_BIT);
    }

    #[cfg(feature = "irq")]
    {
        use super::irq;
        use super::irq::register_handler;

        const GPIO_IRQ_NUMBER: usize = 39;

        info!("Registering GPIO IRQ handler at IRQ number {}", GPIO_IRQ_NUMBER);
        register_handler(GPIO_IRQ_NUMBER, gpio_interrupt_handler);
        irq::set_enable(GPIO_IRQ_NUMBER, true);
    }

    info!("GPIO interrupt setup completed");
}

pub fn gpio_interrupt_handler() {
    use core::arch::asm;

    let gpio_phys_addr = pa!(axconfig::devices::GPIO_PADDR);
    let gpio_virt_addr = phys_to_virt(gpio_phys_addr).as_mut_ptr();

    // GPIO 中断清除寄存器偏移
    const GPIO_INTERRUPT_CLEAR_OFFSET: usize = 0x41c;
    const GPIO_PIN3_CLEAR_BIT: u32 = 1 << 3;

    info!("GPIO interrupt triggered: executing power off sequence");

    unsafe {
        let interrupt_clear_reg = gpio_virt_addr.add(GPIO_INTERRUPT_CLEAR_OFFSET) as *mut u32;
        let current_val = core::ptr::read_volatile(interrupt_clear_reg);
        info!("GPIO Interrupt Clear Register before clearing: {:#x}", current_val);

        // 清除 GPIO3 中断标志位
        core::ptr::write_volatile(interrupt_clear_reg, GPIO_PIN3_CLEAR_BIT);

        // 触发关机指令
        info!("Bye~");
        // asm!("mov w0, #0x18");
        // asm!("hlt #0xF000");
        terminate();
    }
}
