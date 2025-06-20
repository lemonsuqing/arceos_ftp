#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

// 引入 arceos_api 模块
use arceos_api;
use axdriver::drivers::pl061_poweroff;

#[cfg_attr(feature = "axstd", unsafe(no_mangle))]
fn main() {
    // 调用系统初始化，完成平台和驱动初始化
    arceos_api::system_init();

    pl061_poweroff::gpio_poweroff_init();

    // 打印Hello信息
    println!("Hello, world!");
}
