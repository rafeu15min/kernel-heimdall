#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
// Importamos o enum Color para facilitar a digitação
use crate::vga_buffer::Color;

mod gdt;
mod interrupts;
mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Podemos até usar o vermelho no pânico agora!
    println!(fg: Color::LightRed, "!!! KERNEL PANIC !!!");
    println!(fg: Color::LightRed, "{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    print!("Inicializando o Kernel ");
    print!(fg: Color::LightCyan, "Heimdall");
    println!("...");

    gdt::init();
    interrupts::init_idt();

    // 1. Inicia o chip controlador PIC
    unsafe { interrupts::PICS.lock().initialize() };

    // 2. Abre as comportas do processador (Instrução 'sti')
    x86_64::instructions::interrupts::enable();

    println!("Hardware ativado. Escutando interrupcoes externas...");

    // O loop infinito final (O Kernel fica ocioso aguardando teclas)
    loop {
        // Halt pausa a CPU (economizando energia) até a próxima interrupção chegar!
        x86_64::instructions::hlt();
    }
}
