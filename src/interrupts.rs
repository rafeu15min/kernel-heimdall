use crate::{gdt, println}; // Importe o gdt aqui!
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Enum para mapear os fios físicos da placa-mãe (IRQs)
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ));

    // Assim como o VGA, a IDT precisa ser global e estática
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // Cadastramos a nossa função personalizada para lidar com o choque do Breakpoint
        idt.breakpoint.set_handler_fn(breakpoint_handler);

        // Sem essas linhas, a GDT e o handler ficam isolados e geram os avisos.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_u8()]
                    .set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.as_u8()]
            .set_handler_fn(keyboard_interrupt_handler);
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

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    crate::print!(fg: crate::vga_buffer::Color::Red, "\n[FALHA CATASTRÓFICA] ");
    crate::println!(fg: crate::vga_buffer::Color::White, "DOUBLE FAULT");
    crate::println!("{:#?}", stack_frame);

    loop {}
}

// --- TRATADOR DO RELÓGIO (Timer) ---
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Imprime um ponto a cada tique do relógio
    crate::print!(".");

    // Regra de Ouro: Avisa o PIC que terminamos de processar
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// --- TRATADOR DO TECLADO ---
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    // Lemos a Porta de hardware 0x60 (onde o teclado cospe os elétrons)
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();

    // Passamos o pulso elétrico para a máquina de estado traduzir para ASCII
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => crate::print!("{}", character),
                DecodedKey::RawKey(key) => crate::print!("{:?}", key),
            }
        }
    }

    // Regra de Ouro: Libera o teclado para a próxima tecla
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
