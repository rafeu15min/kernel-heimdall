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
    // O texto normal continua saindo em amarelo
    print!("Inicializando o Kernel ");

    // Injetamos a cor apenas na palavra Heimdall!
    print!(fg: Color::LightCyan, "Heimdall");

    // Como a cor é restaurada automaticamente, o resto volta a ser amarelo natural
    println!("...");

    // 1. O Heimdall envia a Tabela de Interrupções para o Hardware
    gdt::init();
    interrupts::init_idt();
    println!("Tabela de Interrupcoes (IDT) carregada com sucesso.");

    // 2. Disparamos o Breakpoint manualmente direto no silício!
    // Isso emula o processador gritando por causa de um choque.
    x86_64::instructions::interrupts::int3();

    // 3. Se a IDT funcionar, o processador vai pausar, exibir a mensagem amarela
    // do nosso handler, e DEPOIS VOLTAR PARA ESTA LINHA e continuar rodando!
    println!(fg: Color::LightGreen, "Sobrevivemos ao Breakpoint! O Kernel continua rodando.");

    loop {}
}
