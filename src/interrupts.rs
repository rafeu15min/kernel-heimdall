use crate::{gdt, println}; // Importe o gdt aqui!
use core::sync::atomic::{AtomicU8, Ordering};
use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1, layouts};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// Começamos no 1 (Azul) para pular o 0 (Preto, senão a palavra some)
static COLOR_TICK: AtomicU8 = AtomicU8::new(1);

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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // 1. Pega o número da cor atual e soma 1 atomicamente
    let tick = COLOR_TICK.fetch_add(1, Ordering::Relaxed);

    // 2. Fazemos uma matemática simples com resto da divisão (módulo % 15).
    // Isso garante que a cor sempre vai girar entre 1 (Azul) e 15 (Branco).
    let next_color = (tick % 15) + 1;

    // 3. Chamamos o nosso pincel cirúrgico lá no VGA
    crate::vga_buffer::rgb_heimdall_effect(next_color);

    // 4. Regra de Ouro: Avisa o PIC
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
        // Passamos o pulso elétrico para a máquina de estado traduzir
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    // Se for um caractere imprimível
                    DecodedKey::Unicode(character) => match character {
                        // O Backspace no padrão ASCII é o código hexadecimal \x08
                        '\x08' => crate::vga_buffer::WRITER.lock().backspace(),

                        // Qualquer outra letra, imprimimos normalmente
                        _ => crate::print!("{}", character),
                    },

                    // Se for uma tecla de controle (RawKey)
                    DecodedKey::RawKey(key) => {
                        let mut writer = crate::vga_buffer::WRITER.lock();
                        match key {
                            KeyCode::ArrowLeft => writer.move_cursor_left(),
                            KeyCode::ArrowRight => writer.move_cursor_right(),

                            // O GRANDE TRUQUE: O underscore '_' captura todo o resto.
                            // Shifts, Controls, CapsLock, F1, Esc...
                            // Ao usar um bloco vazio {}, nós silenciamos essas teclas!
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Regra de Ouro: Libera o teclado para a próxima tecla
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
