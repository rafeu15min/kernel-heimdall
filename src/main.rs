#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Esta função é chamada se o Kernel sofrer um panic!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {} // Trava a CPU em um loop infinito seguro
}

// O ponto de entrada real invocado pelo bootloader
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Endereço de memória física padronizado do buffer de texto VGA
    let vga_buffer = 0xb8000 as *mut u8;

    // Nosso array de bytes estático (gravado direto no binário)
    let mensagem = b"Ola, Engenheiro! O Kernel respira!";
    let cor_verde = 0xa;

    // Iteramos sobre a mensagem pegando o índice (i) e a letra (byte)
    for (i, &byte) in mensagem.iter().enumerate() {
        unsafe {
            // A matemática do VGA: cada caractere ocupa 2 bytes (Letra + Cor)
            // offset(0) = Letra 1 | offset(1) = Cor 1
            // offset(2) = Letra 2 | offset(3) = Cor 2
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = cor_verde;
        }
    }

    loop {} // Mantém a CPU ligada e o Kernel rodando
}
