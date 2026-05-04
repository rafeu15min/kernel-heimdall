use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

// O índice do nosso "Paraquedas" (Double Fault) na tabela de pilhas do TSS
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // 1. Criando o TSS (Task State Segment)
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Reservamos uma pilha de memória exclusiva para erros catastróficos.
        // Se o Kernel corromper a memória principal, ele usa essa área isolada para avisar o erro.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // 20 KB de memória (5 páginas)
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr({ &raw const STACK });
            // Pilhas em x86 crescem de cima para baixo, então retornamos o final do endereço
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };
}

lazy_static! {
    // 2. Criando a GDT (Global Descriptor Table)
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Definimos que este é o código do Kernel (Privilégio Máximo - Ring 0)
        let code_selector = gdt.append(Descriptor::kernel_code_segment());

        // Anexamos o nosso paraquedas na GDT
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors { code_selector, tss_selector })
    };
}

// Uma estrutura simples para guardar as chaves de acesso geradas pela GDT
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// A função que injeta nossa GDT no silício
pub fn init() {
    use x86_64::instructions::segmentation::{CS, Segment};
    use x86_64::instructions::tables::load_tss;

    // Carrega a GDT física no processador
    GDT.0.load();

    // O Rust exige 'unsafe' aqui porque estamos alterando os registradores
    // vitais do processador em tempo real. Um passo em falso e o PC reinicia.
    unsafe {
        // Atualiza o registrador de Segmento de Código (CS) para apontar para nossa nova GDT
        CS::set_reg(GDT.1.code_selector);
        // Informa ao processador onde está o TSS
        load_tss(GDT.1.tss_selector);
    }
}
