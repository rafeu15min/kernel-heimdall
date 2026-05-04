use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    // Assim como o VGA, a IDT precisa ser global e estática
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Cadastramos a nossa função personalizada para lidar com o choque do Breakpoint
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        idt
    };
}

// Essa função publica o carregamento da IDT para o processador
pub fn init_idt() {
    IDT.load();
}

// O Tratador da Exceção (A Mágica)
// O processador nos entrega de bandeja o "InterruptStackFrame", que contém
// o exato estado dos registradores da CPU no nanossegundo em que o erro ocorreu!
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    // Usamos a nossa macro global recém-criada para exibir o erro
    println!("EXCECAO CAPTURADA: BREAKPOINT");
    println!("{:#?}", stack_frame);
}
