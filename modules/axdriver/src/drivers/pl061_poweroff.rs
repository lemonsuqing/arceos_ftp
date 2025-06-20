
use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

// Qemu virt平台PL061 GPIO基地址（请根据实际平台修改）
const GPIO_BASE: usize = 0x0900_0000;

// 寄存器偏移
const GPIORIS_OFFSET: usize = 0x10;   // 原始中断状态寄存器
const GPIOIE_OFFSET: usize = 0x14;    // 中断掩码寄存器
const GPIOIC_OFFSET: usize = 0x1C;    // 中断清除寄存器
const GPIOICFGR_OFFSET: usize = 0x0C; // 中断配置寄存器（ICFGR）

// GPIO寄存器地址
fn gpio_reg(offset: usize) -> *mut u32 {
    (GPIO_BASE + offset) as *mut u32
}

// 第3号GPIO线对应位
const GPIO_LINE_3: u32 = 1 << 3;

// 中断号
const IRQ_GPIO: usize = 39;

// 读寄存器
fn gpio_read(offset: usize) -> u32 {
    unsafe { read_volatile(gpio_reg(offset)) }
}

// 写寄存器
fn gpio_write(offset: usize, val: u32) {
    unsafe { write_volatile(gpio_reg(offset), val) }
}

/// 初始化GPIO关机中断
pub fn gpio_poweroff_init() {
    // 设置第3号线为电平触发（ICFGR寄存器对应位清0）
    let mut icfgr = gpio_read(GPIOICFGR_OFFSET);
    icfgr &= !GPIO_LINE_3;
    gpio_write(GPIOICFGR_OFFSET, icfgr);

    // 清除第3号线中断请求
    gpio_write(GPIOIC_OFFSET, GPIO_LINE_3);

    // 使能第3号线中断
    let mut ie = gpio_read(GPIOIE_OFFSET);
    ie |= GPIO_LINE_3;
    gpio_write(GPIOIE_OFFSET, ie);

    // 注册中断处理函数（假设有此函数，需根据ArceOS中断框架实现）
    register_irq_handler(IRQ_GPIO, gpio_poweroff_irq_handler);

    // 设置中断优先级为0（根据平台具体API）
    set_irq_priority(IRQ_GPIO, 0);

    // 设置中断目标核为0（单核或指定核）
    set_irq_target_cpu(IRQ_GPIO, 0);

    // 使能中断（根据平台中断控制器实现）
    enable_irq(IRQ_GPIO);
}

/// GPIO中断处理函数
fn gpio_poweroff_irq_handler() {
    // 判断是否是第3号线中断
    let ris = gpio_read(GPIORIS_OFFSET);
    if (ris & GPIO_LINE_3) != 0 {
        // 清除中断
        gpio_write(GPIOIC_OFFSET, GPIO_LINE_3);

        // 执行关机
        system_poweroff();
    }
}

/// 执行关机的汇编指令
#[inline(always)]
fn system_poweroff() -> ! {
    unsafe {
        asm!(
            "mov w0, #0x18",
            "hlt #0xF000",
            options(noreturn)
        );
    }
}

/// 以下为中断控制器相关函数的示例声明，需根据ArceOS实际中断框架实现
use axhal::irq;

fn register_irq_handler(irq: usize, handler: fn()) {
    // 这里调用ArceOS中断注册接口，示意用
    // 例如：axhal::irq::register_handler(irq, handler);
    irq::register_handler(irq, handler);
}

fn set_irq_priority(_irq: usize, _priority: u8) {
    // 设置GIC中断优先级，示意用
    // irq::set_priority(irq, priority);
}

fn set_irq_target_cpu(_irq: usize, _cpu: u8) {
    // 设置中断目标CPU，示意用
    // irq::set_target(irq, cpu);
}

fn enable_irq(irq: usize) {
    // 使能中断，示意用
    irq::set_enable(irq, true);
}
