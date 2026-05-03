#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

mod vga_buffer;

// Esta função é chamada se o Kernel sofrer um panic!
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {} // Trava a CPU em um loop infinito seguro
}

// O ponto de entrada real invocado pelo bootloader
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // Apenas UM motor é necessário
    let mut motor_vga = vga_buffer::Writer {
        column_position: 0,
        row_position: 0,
        color_code: vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut vga_buffer::Buffer) },
    };

    let cores_arco_iris = [
        vga_buffer::Color::Blue,
        vga_buffer::Color::Green,
        vga_buffer::Color::Cyan,
        vga_buffer::Color::Red,
        vga_buffer::Color::Magenta,
        vga_buffer::Color::Brown,
        vga_buffer::Color::LightGray,
        vga_buffer::Color::DarkGray,
        vga_buffer::Color::LightBlue,
        vga_buffer::Color::LightGreen,
        vga_buffer::Color::LightCyan,
        vga_buffer::Color::LightRed,
        vga_buffer::Color::Pink,
        vga_buffer::Color::Yellow,
        vga_buffer::Color::White,
    ];

    let mut indice_cor = 0;

    loop {
        // Rebobina o cursor a cada ciclo de animação
        motor_vga.column_position = 0;
        motor_vga.row_position = 0;

        // 1. Configura para Amarelo e escreve o prefixo (com espaço no final)
        motor_vga.color_code =
            vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        let _ = write!(motor_vga, "O Motor do seu ");

        // 2. Troca a cor do mesmo motor para a cor atual do arco-íris e escreve "Kernel"
        motor_vga.color_code =
            vga_buffer::ColorCode::new(cores_arco_iris[indice_cor], vga_buffer::Color::Black);
        let _ = write!(motor_vga, "Kernel");

        // 3. Volta imediatamente para Amarelo e finaliza a linha (com espaço no começo)
        motor_vga.color_code =
            vga_buffer::ColorCode::new(vga_buffer::Color::Yellow, vga_buffer::Color::Black);
        let _ = write!(motor_vga, " respira.\n");

        // Continua imprimindo o resto normalmente em Amarelo
        let _ = write!(motor_vga, "Iniciando subsistemas...\n");
        let _ = write!(motor_vga, "Quantidade de nucleos encontrados: {}\n", 8);
        let _ = write!(
            motor_vga,
            "Memoria RAM fisica enderecada: {} Gigabytes\n",
            32
        );

        // O delay (Busy Wait)
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }

        // Avança a cor
        indice_cor = (indice_cor + 1) % cores_arco_iris.len();
    }
}
