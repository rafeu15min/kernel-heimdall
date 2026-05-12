use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::port::Port;

// O hardware VGA suporta essas 16 cores básicas.
// #[repr(u8)] garante que cada cor ocupe exatamente 1 byte (8 bits) na memória.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// Representa um byte de cor inteiro (4 bits para fundo, 4 bits para letra)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Representa exatamente como a placa de vídeo exige que uma letra seja montada na RAM
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// O tamanho padrão da tela VGA é 80 colunas por 25 linhas
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// O Buffer em si. Usamos Volatile para o compilador não deletar nossas escritas!
#[repr(transparent)]
pub struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// O nosso motorista do VGA: Ele lembra onde o cursor parou e qual a cor atual
pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    pub color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    // Função interna para escrever apenas UM caractere (uma letra)
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // Se for Enter, pula linha
            byte => {
                // Se a linha encheu, pula linha automaticamente
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;
                let color_code = self.color_code;

                // Escreve fisicamente na memória!
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;

                // ADICIONE ESTA LINHA: Para o cursor piscar na próxima casa vazia!
                self.update_hardware_cursor(self.row_position, self.column_position);
            }
        }
    }

    // Adicione isso DENTRO do `impl Writer`
    pub fn change_color_at(&mut self, row: usize, col_start: usize, len: usize, color: Color) {
        let color_code = ColorCode::new(color, Color::Black);

        for i in 0..len {
            let col = col_start + i;
            if col < BUFFER_WIDTH {
                // Lemos o caractere atual da tela
                let mut screen_char = self.buffer.chars[row][col].read();
                // Trocamos apenas a cor dele
                screen_char.color_code = color_code;
                // Escrevemos de volta no hardware
                self.buffer.chars[row][col].write(screen_char);
            }
        }
    }
    // 1. Comunica-se com a placa de vídeo para mover o tracinho piscante
    fn update_hardware_cursor(&self, row: usize, col: usize) {
        let pos = (row * BUFFER_WIDTH + col) as u16;
        unsafe {
            // A porta 0x3D4 recebe o comando (14 para byte alto, 15 para byte baixo)
            // A porta 0x3D5 recebe o dado real da posição
            let mut port_3d4 = Port::new(0x3D4);
            let mut port_3d5 = Port::new(0x3D5);

            port_3d4.write(0x0Fu8);
            port_3d5.write((pos & 0xFF) as u8);

            port_3d4.write(0x0Eu8);
            port_3d5.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    // 2. A mecânica do Backspace
    pub fn backspace(&mut self) {
        if self.column_position > 0 {
            // Volta uma casa
            self.column_position -= 1;

            // Cria um caractere vazio (espaço) com a cor atual
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };

            // Usamos a linha ATUAL (self.row_position) para apagar
            let row = self.row_position;
            let col = self.column_position;

            self.buffer.chars[row][col].write(blank);
            self.update_hardware_cursor(row, col);
        }
    }

    // 3. A mecânica das Setinhas (Navegação Esquerda/Direita)
    // Nota: Em um terminal simples, escrevemos sempre na última linha.
    // Setas para cima/baixo exigiriam histórico de comandos, então faremos a navegação lateral.
    pub fn move_cursor_left(&mut self) {
        if self.column_position > 0 {
            self.column_position -= 1;
            self.update_hardware_cursor(self.row_position, self.column_position);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.column_position < BUFFER_WIDTH - 1 {
            self.column_position += 1;
            self.update_hardware_cursor(self.row_position, self.column_position);
        }
    }
    // A função simplificada de pular linha (Para a versão 1, ela apenas zera a coluna)
    // No futuro, implementaremos a rolagem da tela (scroll) aqui.
    fn new_line(&mut self) {
        self.column_position = 0;
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            // O código de rolar a tela inteira pra cima (Scroll) vai entrar aqui no futuro!
        }
    }

    // Escreve uma string inteira pegando byte por byte
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            // Suporta apenas caracteres ASCII válidos (ou joga o bloco '■' pro resto)
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }
}

// A MÁGICA: Ao implementar a interface 'fmt::Write', nós ganhamos o poder de usar
// as macros nativas do Rust (como write!) e formatar números e variáveis.
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// 1. O ESCRITOR GLOBAL SEGURO
lazy_static! {
    // WRITER é uma variável global, estática, protegida contra colisões de múltiplos núcleos.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Modela o cursor de hardware para parecer um "underline" ( _ )
pub fn init_cursor() {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port_3d4 = Port::new(0x3D4);
        let mut port_3d5 = Port::new(0x3D5);

        // Registrador 0x0A: Define a linha de pixel de INÍCIO do cursor
        port_3d4.write(0x0Au8);
        let start: u8 = port_3d5.read();
        port_3d5.write((start & 0xC0) | 14); // Começa na linha de pixel 14

        // Registrador 0x0B: Define a linha de pixel de FIM do cursor
        port_3d4.write(0x0Bu8);
        let end: u8 = port_3d5.read();
        port_3d5.write((end & 0xE0) | 15); // Termina na linha de pixel 15
    }
}

// 2. A FUNÇÃO PONTE
// As macros não conseguem chamar os métodos diretamente de forma elegante,
// então criamos essa função escondida que trava o Mutex e escreve a string formatada.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Usamos o lock() para garantir exclusividade na memória de vídeo
    WRITER.lock().write_fmt(args).unwrap();
}

#[doc(hidden)]
pub fn _print_with_color(foreground: Color, args: fmt::Arguments) {
    // Travamos o Mutex para garantir que nenhum outro núcleo do processador interrompa a pintura
    let mut writer = WRITER.lock();

    // Salva a cor antiga
    let old_color = writer.color_code;

    // Aplica a nova cor de letra (mantendo o fundo preto)
    writer.color_code = ColorCode::new(foreground, Color::Black);

    // Escreve o texto
    writer.write_fmt(args).unwrap();

    // Restaura a cor antiga imediatamente após escrever!
    writer.color_code = old_color;
}

#[macro_export]
macro_rules! print {
    // Nova regra: se começar com 'fg: Cor', chama a função colorida
    (fg: $color:expr, $($arg:tt)*) => {
        $crate::vga_buffer::_print_with_color($color, format_args!($($arg)*));
    };
    // Regra padrão: continua funcionando como antes
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    // Nova regra com cor (repassa para o print! colorido e adiciona a quebra de linha)
    (fg: $color:expr, $($arg:tt)*) => ($crate::print!(fg: $color, "{}\n", format_args!($($arg)*)));
    // Regra padrão
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Adicione isso no final do arquivo vga_buffer.rs
pub fn rgb_heimdall_effect(color_value: u8) {
    // Transmuta o número (0 a 15) direto para o Enum Color.
    // Como definimos o Color com #[repr(u8)], isso é matematicamente seguro!
    let color: Color = unsafe { core::mem::transmute(color_value) };

    // A palavra Heimdall está na Linha 0, começa na Coluna 23, e tem 8 letras.
    WRITER.lock().change_color_at(0, 23, 8, color);
}
