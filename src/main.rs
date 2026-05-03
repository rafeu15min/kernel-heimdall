#![no_std]
#![no_main]

use core::panic::PanicInfo;

mod vga_buffer;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Como agora temos um println! global, podemos imprimir a mensagem de erro exata
    // com o arquivo e linha que causaram o pânico no sistema!
    println!("{}", info);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Não precisamos mais instanciar nada. As macros nativas voltaram à vida.
    println!("Inicializando o Kernel Heimdall...");
    println!("Memoria de hardware abstrata: OK");
    println!("Testando formatacao numerica: {} e {}", 42, 3.14); // Ponto flutuante agora funciona via software!

    // Até o nosso tratador de pânico agora reage como um sistema moderno
    // Descomente a linha abaixo para ver a captura da falha em ação:
    // panic!("Falha critica forjada para teste do sistema.");

    loop {}
}
