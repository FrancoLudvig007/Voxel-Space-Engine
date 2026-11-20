//! # pyrsutils — v1.5.0
//!
//! Biblioteca única de utilidades (zero dependências externas, 100% `std`) que
//! traz para o Rust a ergonomia do dia a dia de Python (e, na seção de
//! "classes", a sintaxe familiar de C++), sem abrir mão de nenhuma garantia do
//! Rust: tipagem estática, checagem em tempo de compilação, *zero-cost
//! abstractions*, propriedade de memória (*ownership*) segura, sem coletor de
//! lixo (GC) e sem custo oculto em tempo de execução. Tudo aqui utiliza
//! **monomorphization** (código especializado por tipo em tempo de compilação)
//! — nunca *dynamic dispatch* (`dyn Any`) escondido.
//!
//! ## O que mudou na v1.5.0 (Cyber Security, Red/Purple Team & Advanced Networking)
//!
//! 1. **Extensão de `net_utils` para Exploit Dev (`pwntools` style):**
//!    Adicionados `sendline`, `recvuntil` (sem *over-read*), `recvline`,
//!    `recvline_string`, `sendafter`, `sendlineafter`, `try_clone` e `interactive`
//!    bidirecional em `TcpClient`.
//! 2. **Sub-redes CIDR IPv4 (`ipaddress`) e Port Ranges (`nmap`):**
//!    Adicionados `Ipv4Cidr`, `parse_cidr` (iterador *lazy* $O(1)$ em memória) e
//!    `parse_ports` (ex.: `"22,80,443,8000-8005"`).
//! 3. **Scanner TCP e Banner Grabber concorrente (`std::thread::scope`):**
//!    Adicionados `PortScanResult`, `tcp_probe` e `tcp_scan` multithread sem
//!    dependências externas.
//! 4. **Packet Crafting (`scapy` style) e Gerador `.pcap` Nativo (Purple Team):**
//!    Adicionados `rfc1071_checksum`, `ipv4_tcp_checksum`, `Ipv4TcpPacket` (montagem
//!    e parse de cabeçalhos IPv4/TCP) e `PcapWriter` (geração nativa de arquivos
//!    `.pcap` `LINKTYPE_RAW` e `LINKTYPE_ETHERNET` para validação de regras
//!    Suricata/Snort/Zeek).
//! 5. **Living off the Land (Linux `/proc`):**
//!    Adicionados `proc_net_tcp`, `parse_proc_net_tcp_str`, `proc_net_arp` e
//!    `parse_proc_net_arp_str` para inspeção silenciosa da pilha de rede do kernel.
//! 6. **Novo módulo `sec_utils` (Seção 12):**
//!    Packing/unpacking de inteiros (`p8`..`p64`, `u8_le`..`u64_le`, `p16_be`..`u64_be`),
//!    `hexdump` estilo `xxd -C`, `hex_encode`/`hex_decode`, `base64_encode`/`base64_decode`,
//!    `xor_cyclic`/`xor_single`, cálculo de entropia de Shannon (`shannon_entropy`,
//!    `is_high_entropy`), sequências De Bruijn (`cyclic`, `cyclic_find`),
//!    `constant_time_eq` e `secure_zero`.
//! 7. **Novas Macros de Segurança e Rede:**
//!    `unhex!`, `tcp_probe!` e `tcp_server_spawn!`.
//!
//! ## O que mudou na v1.4.0 (Reconstrução Sênior)
//!
//! 1. **Higiene total de macros (`$crate`) e compatibilidade entre Editions:**
//!    `input!`, `try_input!` e `timeit!` agora resolvem caminhos via `$crate::`,
//!    e `TryInto` é importado explicitamente para compilar limpo até mesmo se
//!    invocado com `rustc` direto sem `--edition=2021`.
//! 2. **`arithmetic::pow` em $O(\log N)$ e `usize`/`isize`:** Substituído o laço
//!    linear por exponenciação rápida por quadratura; adicionados `usize`,
//!    `isize`, `abs` e `divmod` seguro; corrigido o domínio matemático de
//!    `root(x, indice)` para índices fracionários e zero/NaN.
//! 3. **`string_utils::join` com alocação única + novos utilitários:** Eliminado
//!    o `Vec<String>` intermediário no `join`. Adicionados `replace`, `lower`,
//!    `upper`, `startswith`, `endswith`, `find`, `count` e `is_palindrome`.
//! 4. **Segurança e performance em `fs_utils::walk` + I/O direto:** `Walk` agora
//!    usa `DirEntry::file_type()` (não segue links simbólicos, evitando loops
//!    infinitos e poupando chamadas `stat` ao kernel). Adicionados `read_text`,
//!    `readlines`, `write_text` e `append_text`.
//! 5. **Built-ins expandidos:** `PyLen` agora cobre `String`, `Vec`, `VecDeque`,
//!    `HashSet`, `BTreeMap` e `BTreeSet`. Novas funções: `print`, `enumerate`,
//!    `zip`, `any`, `all` e `histogram` (contagem de frequências estilo Python).
//! 6. **Multiplicação de matrizes otimizada para cache (`Matrix::dot`):**
//!    Transpõe a segunda matriz internamente para garantir leitura sequencial
//!    alinhada na memória contígua.
//! 7. **Macro `class!` com suporte a Herança (`struct Filha : Pai`):**
//!    Implementa relação *IS-A* / *HAS-A* via composição (`pub base: Pai`) +
//!    `Deref`/`DerefMut` com custo zero em tempo de execução.
//! 8. **Novo módulo `random_utils` (Seção 11):** Equivalente ao módulo `random`
//!    do Python (`random`, `randint`, `choice`, `shuffle`, `seed`) usando
//!    SplitMix64 + Xorshift64* em `std` pura.
//!
//! ## Tabela Geral — pyrsutils ↔ Python ↔ C++
//!
//! | Seção      | Python                                      | C++ / Rust nativo                  | Aqui (`pyrsutils`)                                      |
//! |------------|---------------------------------------------|------------------------------------|---------------------------------------------------------|
//! | Entrada    | `input()`                                   | `std::cin >>`                      | `input::<T>(prompt)` / `input!()` / `try_input()`       |
//! | Ranges     | `range(a, b, c)`                            | `for(int i=a;i<b;i+=c)`            | `range` / `range_from` / `range_step`                   |
//! | Listas     | `append/pop/remove/extend/index`            | `std::vector::push_back`           | `Vec<T>` + trait `PyList`                               |
//! | Matriz     | `[[0]*c for _ in range(l)]`                 | `std::vector<vector<T>>`           | `Matrix<T>` (contígua, `transpose`, `dot`)              |
//! | Built-ins  | `print/len/min/max/sorted/enumerate/zip`    | `.size()/std::min/std::sort`       | `builtins::{print,len,min,max,sorted,reversed,zip,...}` |
//! | Aritmética | `sum/math.prod/pow/divmod/abs`              | operadores nativos                 | `arithmetic::{sum,sub,mult,div,divmod,pow,root,abs}`    |
//! | Strings    | `split/strip/join/replace/find/count`       | `std::string`                      | `string_utils::{split,strip,join,replace,find,count}`   |
//! | Classe     | `class Y(X): def __init__(self):`           | `class Y : public X { ... };`      | `class! { struct Y : X {...} impl {...} }`              |
//! | Tempo      | `time/sleep/perf_counter/timeit/strftime`   | `std::chrono` / `clock_gettime`    | `time_utils::{time,sleep,perf_counter,timeit,strftime}` |
//! | Rede       | `socket` / `pwntools` / `scapy` / `nmap`    | `socket()/bind()/accept()/pcap`    | `net_utils::{TcpClient,Ipv4Cidr,tcp_scan,PcapWriter}`   |
//! | Arquivos   | `os/os.path/shutil/open().read()`           | `std::filesystem` / `std::fstream` | `fs_utils::{exists,listdir,walk,glob,read_text,...}`    |
//! | Aleatório  | `random.random/randint/choice/shuffle/seed` | `<random>` (`std::mt19937_64`)     | `random_utils::{random,randint,choice,shuffle,seed}`    |
//! | Segurança  | `pwntools` / `struct` / `binascii`          | `memcpy` / `xxd` / `openssl`       | `sec_utils::{p64,u64_le,hexdump,xor_cyclic,cyclic,...}` |
//!
//! ## Uso Rápido
//! ```rust
//! use pyrsutils::prelude::*;
//! ```

#![allow(unused)]

use std::fmt::{Debug, Display};
use std::io::{self, Write};
use std::str::FromStr;

// ============================================================================
// 0) input() / try_input() — equivalente unificado e genérico ao input()
//    do Python, para qualquer T: FromStr (String, i32, f64, bool, char...)
// ============================================================================

/// Erro unificado de leitura e conversão de entrada padrão (`stdin`).
#[derive(Debug)]
pub enum InputError<E> {
    /// Stream fechado antes de qualquer dado (Ctrl+D no Linux / Ctrl+Z no Windows).
    Eof,
    /// Falha de I/O ao ler `stdin` ou descarregar `stdout`.
    Io(io::Error),
    /// O texto lido não pôde ser convertido para o tipo `T` esperado.
    Parse(E),
}

impl<E: Display> Display for InputError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Eof => write!(f, "EOFError: fim da entrada (Ctrl+D/Ctrl+Z)"),
            InputError::Io(e) => write!(f, "IOError: falha ao ler entrada ({e})"),
            InputError::Parse(e) => write!(f, "ValueError: conversão falhou ({e})"),
        }
    }
}

impl<E: Debug + Display> std::error::Error for InputError<E> {}

/// Lê uma linha crua de `stdin` removendo apenas `\n` ou `\r\n` final.
fn read_raw(prompt: &str) -> Result<String, io::Error> {
    if !prompt.is_empty() {
        print!("{prompt}");
        io::stdout().flush()?;
    }

    let mut buffer = String::new();
    let bytes_read = io::stdin().read_line(&mut buffer)?;

    if bytes_read == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
    }

    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }

    Ok(buffer)
}

/// Equivalente unificado ao `input()` (e `int(input())`, `float(input())`) do Python.
///
/// O tipo de retorno `T` é inferido automaticamente pelo compilador a partir
/// da variável ou função onde o resultado é usado. Se o usuário digitar um
/// valor numérico com espaços nas pontas (ex.: `" 42 "`), para tipos diferentes
/// de `String` o texto passa por `.trim()` automaticamente (igual ao `int(" 42 ")`
/// do Python) caso a conversão direta falhe na primeira tentativa.
///
/// # Panics
/// Encerra com `panic!` em caso de EOF ou falha de conversão (mesmo comportamento
/// de um script Python sem bloco `try/except`). Para tratar erros sem panic,
/// use [`try_input`].
pub fn input<T>(prompt: &str) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    match try_input::<T>(prompt) {
        Ok(valor) => valor,
        Err(InputError::Eof) => panic!("EOFError: entrada finalizada (Ctrl+D/Ctrl+Z)"),
        Err(InputError::Io(e)) => panic!("IOError: erro ao ler stdin: {e}"),
        Err(InputError::Parse(e)) => {
            panic!("ValueError: entrada inválida para o tipo esperado ({e:?})")
        }
    }
}

/// Versão segura de [`input`] que devolve `Result<T, InputError<T::Err>>` em
/// vez de causar `panic!`. Equivalente ao bloco `try: ... except ValueError:`
/// do Python.
pub fn try_input<T>(prompt: &str) -> Result<T, InputError<T::Err>>
where
    T: FromStr,
{
    let texto = read_raw(prompt).map_err(|e| {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            InputError::Eof
        } else {
            InputError::Io(e)
        }
    })?;

    // Tenta o parse exato primeiro (preserva espaços se T = String);
    // se falhar (ex.: " 42 " para i32), tenta com trim() igual ao int() do Python.
    match texto.parse::<T>() {
        Ok(v) => Ok(v),
        Err(_) => texto.trim().parse::<T>().map_err(InputError::Parse),
    }
}

/// Macro ergonômica: `input!()` ou `input!("Digite: ")`.
/// Usa `$crate::input` para funcionar em qualquer módulo sem imports extras.
#[macro_export]
macro_rules! input {
    () => {
        $crate::input("")
    };
    ($prompt:expr) => {
        $crate::input($prompt)
    };
}

/// Macro ergonômica: `try_input!()` ou `try_input!("Digite: ")`.
#[macro_export]
macro_rules! try_input {
    () => {
        $crate::try_input("")
    };
    ($prompt:expr) => {
        $crate::try_input($prompt)
    };
}

// ============================================================================
// 1) range() — sequências lazy sem alocar memória (Iterator O(1) em espaço)
// ============================================================================
pub mod range_utils {
    //! Equivalente ao `range()` do Python para inteiros com sinal (`i64`).
    //!
    //! | Python                     | Aqui                            |
    //! |----------------------------|---------------------------------|
    //! | `range(stop)`              | `range(stop)`                   |
    //! | `range(start, stop)`       | `range_from(start, stop)`       |
    //! | `range(start, stop, step)` | `range_step(start, stop, step)` |

    use super::builtins::PyLen;
    use std::convert::TryInto;

    /// Iterador preguiçoso equivalente ao `range` do Python (suporta passo negativo).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PyRange {
        atual: i64,
        fim: i64,
        passo: i64,
    }

    impl PyRange {
        /// Verifica em $O(1)$ se um valor `x` pertence à sequência (`x in range(...)`).
        #[inline]
        pub fn contains(&self, x: i64) -> bool {
            if self.passo > 0 {
                x >= self.atual && x < self.fim && (x - self.atual) % self.passo == 0
            } else {
                x <= self.atual && x > self.fim && (self.atual - x) % (-self.passo) == 0
            }
        }
    }

    impl Iterator for PyRange {
        type Item = i64;

        #[inline]
        fn next(&mut self) -> Option<i64> {
            if self.passo > 0 {
                if self.atual >= self.fim {
                    return None;
                }
            } else if self.atual <= self.fim {
                return None;
            }
            let valor = self.atual;
            self.atual += self.passo;
            Some(valor)
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let n = ExactSizeIterator::len(self);
            (n, Some(n))
        }
    }

    impl ExactSizeIterator for PyRange {
        #[inline]
        fn len(&self) -> usize {
            let (atual, fim, passo) = (self.atual as i128, self.fim as i128, self.passo as i128);
            let n = if passo > 0 {
                if atual >= fim {
                    0
                } else {
                    (fim - atual + passo - 1) / passo
                }
            } else if atual <= fim {
                0
            } else {
                let passo_abs = -passo;
                (atual - fim + passo_abs - 1) / passo_abs
            };
            n.try_into().unwrap_or(usize::MAX)
        }
    }

    impl PyLen for PyRange {
        #[inline]
        fn py_len(&self) -> usize {
            ExactSizeIterator::len(self)
        }
    }

    /// `range(stop)` — gera `0, 1, ..., stop - 1`.
    #[inline]
    pub fn range(stop: i64) -> PyRange {
        PyRange {
            atual: 0,
            fim: stop,
            passo: 1,
        }
    }

    /// `range_from(start, stop)` — gera `start, start + 1, ..., stop - 1`.
    #[inline]
    pub fn range_from(start: i64, stop: i64) -> PyRange {
        PyRange {
            atual: start,
            fim: stop,
            passo: 1,
        }
    }

    /// `range_step(start, stop, step)` — suporta passo positivo ou negativo.
    ///
    /// # Panics
    /// Causa panic se `step == 0` (`ValueError` no Python).
    #[inline]
    pub fn range_step(start: i64, stop: i64, step: i64) -> PyRange {
        assert!(step != 0, "ValueError: range() com step igual a zero");
        PyRange {
            atual: start,
            fim: stop,
            passo: step,
        }
    }
}

// ============================================================================
// 2) Listas — métodos estilo Python sobre Vec<T>, via Extension Trait
// ============================================================================
pub mod list_utils {
    //! Métodos ergonômicos estilo Python sobre `Vec<T>` com custo zero.

    pub trait PyList<T> {
        /// Adiciona um item ao final — equivalente a `lista.append(x)`.
        fn append_item(&mut self, valor: T);

        /// Estende a lista com qualquer iterável — equivalente a `lista.extend(iter)`.
        fn extend_from<I: IntoIterator<Item = T>>(&mut self, iter: I);

        /// Remove e devolve o item no índice dado, ou o último se `None` —
        /// equivalente a `lista.pop()` / `lista.pop(i)`.
        fn pop_at(&mut self, indice: Option<usize>) -> Option<T>;

        /// Remove a primeira ocorrência igual a `valor` — equivalente a `lista.remove(x)`.
        /// Devolve `true` se encontrou e removeu.
        fn remove_value(&mut self, valor: &T) -> bool
        where
            T: PartialEq;

        /// Devolve o índice da primeira ocorrência de `valor` — equivalente a `lista.index(x)`.
        fn index_of(&self, valor: &T) -> Option<usize>
        where
            T: PartialEq;

        /// Conta quantas vezes `valor` aparece na lista — equivalente a `lista.count(x)`.
        fn count_value(&self, valor: &T) -> usize
        where
            T: PartialEq;
    }

    impl<T> PyList<T> for Vec<T> {
        #[inline]
        fn append_item(&mut self, valor: T) {
            self.push(valor);
        }

        #[inline]
        fn extend_from<I: IntoIterator<Item = T>>(&mut self, iter: I) {
            self.extend(iter);
        }

        #[inline]
        fn pop_at(&mut self, indice: Option<usize>) -> Option<T> {
            match indice {
                None => self.pop(),
                Some(i) if i < self.len() => Some(self.remove(i)),
                Some(_) => None,
            }
        }

        fn remove_value(&mut self, valor: &T) -> bool
        where
            T: PartialEq,
        {
            match self.iter().position(|x| x == valor) {
                Some(pos) => {
                    self.remove(pos);
                    true
                }
                None => false,
            }
        }

        #[inline]
        fn index_of(&self, valor: &T) -> Option<usize>
        where
            T: PartialEq,
        {
            self.iter().position(|x| x == valor)
        }

        #[inline]
        fn count_value(&self, valor: &T) -> usize
        where
            T: PartialEq,
        {
            self.iter().filter(|&x| x == valor).count()
        }
    }
}

// ============================================================================
// 3) Matriz — dados contíguos (cache-friendly em x86-64), não Vec<Vec<T>>
// ============================================================================
pub mod matrix_utils {
    //! Matriz bidimensional densa com armazenamento contíguo (`Vec<T>` linear).
    //! Garante localidade espacial na cache L1/L2 da CPU e habilita auto-vetorização
    //! (SIMD/SSE/AVX) pelo compilador LLVM.

    use super::arithmetic::NumeroPy;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Matrix<T> {
        dados: Vec<T>,
        pub linhas: usize,
        pub colunas: usize,
    }

    impl<T: Clone> Matrix<T> {
        /// Cria uma matriz `linhas x colunas` preenchida com `valor_padrao`.
        /// Equivalente a `[[valor_padrao] * colunas for _ in range(linhas)]`.
        pub fn new(linhas: usize, colunas: usize, valor_padrao: T) -> Self {
            Matrix {
                dados: vec![valor_padrao; linhas.saturating_mul(colunas)],
                linhas,
                colunas,
            }
        }

        /// Cria uma matriz a partir de uma função geradora `f(linha, coluna)`.
        pub fn from_fn<F>(linhas: usize, colunas: usize, mut f: F) -> Self
        where
            F: FnMut(usize, usize) -> T,
        {
            let mut dados = Vec::with_capacity(linhas.saturating_mul(colunas));
            for i in 0..linhas {
                for j in 0..colunas {
                    dados.push(f(i, j));
                }
            }
            Matrix {
                dados,
                linhas,
                colunas,
            }
        }

        /// Acesso seguro com verificação de limites (`Option<&T>`).
        #[inline]
        pub fn get(&self, linha: usize, coluna: usize) -> Option<&T> {
            if linha < self.linhas && coluna < self.colunas {
                Some(&self.dados[linha * self.colunas + coluna])
            } else {
                None
            }
        }

        /// Acesso mutável seguro com verificação de limites (`Option<&mut T>`).
        #[inline]
        pub fn get_mut(&mut self, linha: usize, coluna: usize) -> Option<&mut T> {
            if linha < self.linhas && coluna < self.colunas {
                Some(&mut self.dados[linha * self.colunas + coluna])
            } else {
                None
            }
        }

        /// Devolve uma fatia (`&[T]`) da linha `linha` em $O(1)$, sem alocar.
        #[inline]
        pub fn row(&self, linha: usize) -> Option<&[T]> {
            if linha < self.linhas {
                let ini = linha * self.colunas;
                Some(&self.dados[ini..ini + self.colunas])
            } else {
                None
            }
        }

        /// Devolve a matriz transposta ($M^T$) como uma nova `Matrix<T>`.
        pub fn transpose(&self) -> Matrix<T> {
            if self.dados.is_empty() {
                return Matrix {
                    dados: Vec::new(),
                    linhas: self.colunas,
                    colunas: self.linhas,
                };
            }
            let mut nova = Matrix::new(self.colunas, self.linhas, self.dados[0].clone());
            for i in 0..self.linhas {
                for j in 0..self.colunas {
                    nova[(j, i)] = self[(i, j)].clone();
                }
            }
            nova
        }

        /// Consome a matriz e devolve o `Vec<T>` linear subjacente sem cópia.
        #[inline]
        pub fn flatten(self) -> Vec<T> {
            self.dados
        }
    }

    impl<T: NumeroPy> Matrix<T> {
        /// Multiplicação matricial (`self * other`) otimizada para a cache da CPU.
        ///
        /// Transpõe `other` internamente antes do produto escalar para garantir que
        /// ambos os laços internos percorram a memória de forma estritamente
        /// sequencial (alinhada), facilitando o uso de instruções SIMD pelo LLVM.
        /// Devolve `None` se `self.colunas != other.linhas`.
        pub fn dot(&self, other: &Matrix<T>) -> Option<Matrix<T>> {
            if self.colunas != other.linhas {
                return None;
            }
            let n = self.colunas;
            let other_t = other.transpose();
            let mut res = Matrix::new(self.linhas, other.colunas, T::ZERO);

            for i in 0..self.linhas {
                let linha_a = &self.dados[i * n..(i + 1) * n];
                for j in 0..other.colunas {
                    let linha_bt = &other_t.dados[j * n..(j + 1) * n];
                    let mut soma = T::ZERO;
                    for k in 0..n {
                        soma = soma + linha_a[k] * linha_bt[k];
                    }
                    res[(i, j)] = soma;
                }
            }
            Some(res)
        }
    }

    impl<T> std::ops::Index<(usize, usize)> for Matrix<T> {
        type Output = T;
        #[inline]
        fn index(&self, (linha, coluna): (usize, usize)) -> &T {
            assert!(
                linha < self.linhas && coluna < self.colunas,
                "IndexError: índice ({linha}, {coluna}) fora da matriz {}x{}",
                self.linhas,
                self.colunas
            );
            &self.dados[linha * self.colunas + coluna]
        }
    }

    impl<T> std::ops::IndexMut<(usize, usize)> for Matrix<T> {
        #[inline]
        fn index_mut(&mut self, (linha, coluna): (usize, usize)) -> &mut T {
            assert!(
                linha < self.linhas && coluna < self.colunas,
                "IndexError: índice ({linha}, {coluna}) fora da matriz {}x{}",
                self.linhas,
                self.colunas
            );
            &mut self.dados[linha * self.colunas + coluna]
        }
    }
}

// ============================================================================
// 4) Built-ins gerais: print(), len(), min(), max(), sorted(), reversed(),
//    enumerate(), zip(), any(), all(), histogram(), type_of()
// ============================================================================
pub mod builtins {
    //! Funções livres equivalentes às funções integradas (*built-ins*) do Python.

    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
    use std::fmt::Display;
    use std::hash::Hash;

    /// `print(x)` — exibe qualquer valor que implemente `Display` seguido de quebra
    /// de linha. Ideal para quem está migrando do Python e quer imprimir valores
    /// simples sem precisar da string de formatação de `println!("{x}")`.
    #[inline]
    pub fn print<T: Display>(valor: T) {
        println!("{valor}");
    }

    /// `len(&x)` — devolve o comprimento da coleção ou string.
    #[inline]
    pub fn len<T: ?Sized + PyLen>(x: &T) -> usize {
        x.py_len()
    }

    pub trait PyLen {
        fn py_len(&self) -> usize;
    }

    impl PyLen for str {
        /// Conta caracteres Unicode (`chars().count()`), igual ao `len()` do Python.
        #[inline]
        fn py_len(&self) -> usize {
            self.chars().count()
        }
    }

    impl PyLen for String {
        #[inline]
        fn py_len(&self) -> usize {
            self.chars().count()
        }
    }

    impl<T> PyLen for [T] {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<T> PyLen for Vec<T> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<T> PyLen for VecDeque<T> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<K, V> PyLen for HashMap<K, V> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<T> PyLen for HashSet<T> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<K, V> PyLen for BTreeMap<K, V> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    impl<T> PyLen for BTreeSet<T> {
        #[inline]
        fn py_len(&self) -> usize {
            self.len()
        }
    }

    /// `min(iter)` — devolve `Some(menor)` ou `None` se a sequência for vazia.
    pub fn min<T, I>(iter: I) -> Option<T>
    where
        I: IntoIterator<Item = T>,
        T: PartialOrd,
    {
        iter.into_iter().fold(None, |menor, x| match menor {
            None => Some(x),
            Some(m) => {
                if x < m {
                    Some(x)
                } else {
                    Some(m)
                }
            }
        })
    }

    /// `max(iter)` — devolve `Some(maior)` ou `None` se a sequência for vazia.
    pub fn max<T, I>(iter: I) -> Option<T>
    where
        I: IntoIterator<Item = T>,
        T: PartialOrd,
    {
        iter.into_iter().fold(None, |maior, x| match maior {
            None => Some(x),
            Some(m) => {
                if x > m {
                    Some(x)
                } else {
                    Some(m)
                }
            }
        })
    }

    /// `sorted(iter)` — devolve um novo `Vec<T>` ordenado em ordem crescente.
    pub fn sorted<T, I>(iter: I) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
        T: PartialOrd,
    {
        let mut v: Vec<T> = iter.into_iter().collect();
        v.sort_by(|a, b| {
            a.partial_cmp(b)
                .expect("ValueError: valores incomparáveis em sorted() (ex.: NaN)")
        });
        v
    }

    /// `reversed(iter)` — devolve um iterador reverso preguiçoso (sem alocar).
    #[inline]
    pub fn reversed<I>(iter: I) -> std::iter::Rev<I::IntoIter>
    where
        I: IntoIterator,
        I::IntoIter: DoubleEndedIterator,
    {
        iter.into_iter().rev()
    }

    /// `enumerate(iter)` — produz pares `(indice_usize, elemento)` igual ao Python.
    #[inline]
    pub fn enumerate<I: IntoIterator>(iter: I) -> std::iter::Enumerate<I::IntoIter> {
        iter.into_iter().enumerate()
    }

    /// `zip(a, b)` — caminha por duas sequências em paralelo produzindo tuplas `(a_i, b_i)`.
    #[inline]
    pub fn zip<A: IntoIterator, B: IntoIterator>(
        a: A,
        b: B,
    ) -> std::iter::Zip<A::IntoIter, B::IntoIter> {
        a.into_iter().zip(b)
    }

    /// `any(iter)` — devolve `true` se ao menos um elemento da sequência for `true`.
    #[inline]
    pub fn any<I: IntoIterator<Item = bool>>(iter: I) -> bool {
        iter.into_iter().any(|x| x)
    }

    /// `all(iter)` — devolve `true` se todos os elementos da sequência forem `true`.
    #[inline]
    pub fn all<I: IntoIterator<Item = bool>>(iter: I) -> bool {
        iter.into_iter().all(|x| x)
    }

    /// `histogram(iter)` — equivalente a `collections.Counter(iter)` do Python
    /// (ou à função clássica `histogram` de contagem de frequências em dicionário).
    /// Conta quantas vezes cada item aparece na sequência em tempo $O(N)$.
    pub fn histogram<T, I>(iter: I) -> HashMap<T, usize>
    where
        I: IntoIterator<Item = T>,
        T: Eq + Hash,
    {
        let mut mapa = HashMap::new();
        for item in iter {
            *mapa.entry(item).or_insert(0) += 1;
        }
        mapa
    }

    /// `type_of(&x)` — devolve o nome estático do tipo inferido pelo compilador.
    #[inline]
    pub fn type_of<T>(_: &T) -> &'static str {
        std::any::type_name::<T>()
    }
}

// ============================================================================
// 5) Aritmética — sum(), sub(), mult(), div(), divmod(), pow(), root(), abs()
// ============================================================================
pub mod arithmetic {
    //! Aritmética genérica e protegida contra divisão por zero para todas as
    //! larguras numéricas nativas do Rust (`i8`..`i128`, `isize`, `u8`..`u128`,
    //! `usize`, `f32`, `f64`).

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    pub enum ArithError {
        DivisaoPorZero,
        RaizDeNegativoComIndicePar,
        IndiceDeRaizInvalido,
    }

    pub trait NumeroPy:
        Copy
        + PartialEq
        + PartialOrd
        + std::ops::Add<Output = Self>
        + std::ops::Sub<Output = Self>
        + std::ops::Mul<Output = Self>
        + std::ops::Div<Output = Self>
        + std::ops::Rem<Output = Self>
    {
        const ZERO: Self;
        const UM: Self;
        fn abs_val(self) -> Self;
    }

    macro_rules! impl_numero_py_signed {
        ($($tipo:ty => $zero:expr, $um:expr);+ $(;)?) => {
            $(
                impl NumeroPy for $tipo {
                    const ZERO: Self = $zero;
                    const UM: Self = $um;
                    #[inline]
                    fn abs_val(self) -> Self {
                        if self < Self::ZERO { Self::ZERO - self } else { self }
                    }
                }
            )+
        };
    }

    macro_rules! impl_numero_py_unsigned {
        ($($tipo:ty => $zero:expr, $um:expr);+ $(;)?) => {
            $(
                impl NumeroPy for $tipo {
                    const ZERO: Self = $zero;
                    const UM: Self = $um;
                    #[inline]
                    fn abs_val(self) -> Self {
                        self
                    }
                }
            )+
        };
    }

    impl_numero_py_signed!(
        i8 => 0, 1;
        i16 => 0, 1;
        i32 => 0, 1;
        i64 => 0, 1;
        i128 => 0, 1;
        isize => 0, 1;
        f32 => 0.0, 1.0;
        f64 => 0.0, 1.0;
    );

    impl_numero_py_unsigned!(
        u8 => 0, 1;
        u16 => 0, 1;
        u32 => 0, 1;
        u64 => 0, 1;
        u128 => 0, 1;
        usize => 0, 1;
    );

    /// `abs(x)` — valor absoluto genérico para qualquer tipo numérico.
    #[inline]
    pub fn abs<T: NumeroPy>(x: T) -> T {
        x.abs_val()
    }

    /// `sum(iter)` — soma todos os elementos; devolve `0` se vazio (`sum([]) == 0`).
    #[inline]
    pub fn sum<T: NumeroPy, I: IntoIterator<Item = T>>(iter: I) -> T {
        iter.into_iter().fold(T::ZERO, |acc, x| acc + x)
    }

    /// `sub(iter)` — subtração encadeada (`primeiro - segundo - ...`); `None` se vazio.
    pub fn sub<T: NumeroPy, I: IntoIterator<Item = T>>(iter: I) -> Option<T> {
        let mut it = iter.into_iter();
        let primeiro = it.next()?;
        Some(it.fold(primeiro, |acc, x| acc - x))
    }

    /// `mult(iter)` — produtório (`math.prod` do Python); devolve `1` se vazio.
    #[inline]
    pub fn mult<T: NumeroPy, I: IntoIterator<Item = T>>(iter: I) -> T {
        iter.into_iter().fold(T::UM, |acc, x| acc * x)
    }

    /// `div(a, b)` — divisão segura que devolve `Err(ArithError::DivisaoPorZero)` se `b == 0`.
    #[inline]
    pub fn div<T: NumeroPy>(a: T, b: T) -> Result<T, ArithError> {
        if b == T::ZERO {
            Err(ArithError::DivisaoPorZero)
        } else {
            Ok(a / b)
        }
    }

    /// `divmod(a, b)` — equivalente ao `divmod(a, b)` do Python: devolve a tupla
    /// `(quociente, resto)` em uma única operação segura contra divisão por zero.
    #[inline]
    pub fn divmod<T: NumeroPy>(a: T, b: T) -> Result<(T, T), ArithError> {
        if b == T::ZERO {
            Err(ArithError::DivisaoPorZero)
        } else {
            Ok((a / b, a % b))
        }
    }

    /// `pow(base, expoente)` — exponenciação rápida por quadratura em $O(\log(\text{expoente}))$.
    #[inline]
    pub fn pow<T: NumeroPy>(mut base: T, mut expoente: u32) -> T {
        let mut resultado = T::UM;
        while expoente > 0 {
            if expoente & 1 == 1 {
                resultado = resultado * base;
            }
            expoente >>= 1;
            if expoente > 0 {
                base = base * base;
            }
        }
        resultado
    }

    /// `root(x, indice)` — raiz n-ésima real (`f64`).
    /// Retorna erro explícito se `indice` for `0.0`/`NaN`, ou se `x < 0.0` com
    /// índice par ou fracionário.
    pub fn root(x: f64, indice: f64) -> Result<f64, ArithError> {
        if indice == 0.0 || indice.is_nan() {
            return Err(ArithError::IndiceDeRaizInvalido);
        }
        if x < 0.0 {
            if indice.fract() != 0.0 || (indice as i64) % 2 == 0 {
                return Err(ArithError::RaizDeNegativoComIndicePar);
            }
            return Ok(-((-x).powf(1.0 / indice)));
        }
        Ok(x.powf(1.0 / indice))
    }
}

// ============================================================================
// 6) Strings — split(), strip(), join(), replace(), lower(), upper(),
//    startswith(), endswith(), find(), count(), is_palindrome()
// ============================================================================
pub mod string_utils {
    //! Manipulação de strings com foco em zero-copy (`&str`) onde possível e
    //! alocação única nas operações que geram texto novo.

    /// `split(s, sep)` — fatias emprestadas de `s` (zero alocação de caracteres).
    #[inline]
    pub fn split<'a>(s: &'a str, sep: &str) -> Vec<&'a str> {
        s.split(sep).collect()
    }

    /// `strip(s)` — remove espaços em branco do início e do fim sem alocar.
    #[inline]
    pub fn strip(s: &str) -> &str {
        s.trim()
    }

    /// `join(sep, iter)` — equivalente a `sep.join(iter)` do Python.
    /// Faz uma única alocação de `String`, escrevendo direto no buffer final.
    pub fn join<I, S>(sep: &str, iter: I) -> String
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut it = iter.into_iter();
        let primeiro = match it.next() {
            Some(p) => p,
            None => return String::new(),
        };
        let (min_len, _) = it.size_hint();
        let mut resultado =
            String::with_capacity(primeiro.as_ref().len() + min_len * (sep.len() + 4));
        resultado.push_str(primeiro.as_ref());
        for item in it {
            resultado.push_str(sep);
            resultado.push_str(item.as_ref());
        }
        resultado
    }

    /// `replace(s, de, para)` — equivalente a `s.replace(de, para)` do Python.
    #[inline]
    pub fn replace(s: &str, de: &str, para: &str) -> String {
        s.replace(de, para)
    }

    /// `lower(s)` — equivalente a `s.lower()` do Python.
    #[inline]
    pub fn lower(s: &str) -> String {
        s.to_lowercase()
    }

    /// `upper(s)` — equivalente a `s.upper()` do Python.
    #[inline]
    pub fn upper(s: &str) -> String {
        s.to_uppercase()
    }

    /// `startswith(s, prefixo)` — equivalente a `s.startswith(prefixo)`.
    #[inline]
    pub fn startswith(s: &str, prefixo: &str) -> bool {
        s.starts_with(prefixo)
    }

    /// `endswith(s, sufixo)` — equivalente a `s.endswith(sufixo)`.
    #[inline]
    pub fn endswith(s: &str, sufixo: &str) -> bool {
        s.ends_with(sufixo)
    }

    /// `find(s, sub)` — devolve o índice **em caracteres Unicode** (igual ao
    /// `s.find(sub)` do Python) ou `-1` se não encontrar.
    pub fn find(s: &str, sub: &str) -> i64 {
        match s.find(sub) {
            Some(byte_idx) => s[..byte_idx].chars().count() as i64,
            None => -1,
        }
    }

    /// `count(s, sub)` — conta ocorrências não-sobrepostas de `sub` em `s`
    /// (`s.count(sub)` do Python).
    #[inline]
    pub fn count(s: &str, sub: &str) -> usize {
        s.matches(sub).count()
    }

    /// `is_palindrome(s)` — verifica em $O(N)$ e zero alocação se `s` é um
    /// palíndromo (lê-se igual de trás para frente em caracteres Unicode).
    #[inline]
    pub fn is_palindrome(s: &str) -> bool {
        let mut chars = s.chars();
        while let (Some(a), Some(b)) = (chars.next(), chars.next_back()) {
            if a != b {
                return false;
            }
        }
        true
    }
}

// ============================================================================
// 7) OOP estilo C++/Python — macro `class!` (com suporte a herança `:`)
// ============================================================================

/// Declara uma estrutura e seus métodos em um único bloco estilo C++/Python,
/// gerando construtor `new(...)` automático e suportando herança (`struct Filha : Pai`)
/// via composição + `Deref`/`DerefMut` (custo zero).
#[macro_export]
macro_rules! class {
    // 1) Classe base padrão
    (
        struct $nome:ident {
            $( $campo:ident : $tipo:ty ),* $(,)?
        }
        impl {
            $( fn $metodo:ident ( $($params:tt)* ) $(-> $ret:ty)? $corpo:block )*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $nome {
            $( pub $campo: $tipo ),*
        }

        impl $nome {
            /// Construtor gerado automaticamente a partir dos campos declarados.
            pub fn new( $( $campo: $tipo ),* ) -> Self {
                Self { $( $campo ),* }
            }

            $(
                pub fn $metodo( $($params)* ) $(-> $ret)? $corpo
            )*
        }
    };

    // 2) Classe derivada com herança estilo C++ (`struct Filha : Pai { ... }`)
    (
        struct $nome:ident : $pai:ty {
            $( $campo:ident : $tipo:ty ),* $(,)?
        }
        impl {
            $( fn $metodo:ident ( $($params:tt)* ) $(-> $ret:ty)? $corpo:block )*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $nome {
            pub base: $pai,
            $( pub $campo: $tipo ),*
        }

        impl $nome {
            /// Construtor que recebe a instância da classe `base` seguida dos campos próprios.
            pub fn new(base: $pai, $( $campo: $tipo ),* ) -> Self {
                Self { base, $( $campo ),* }
            }

            $(
                pub fn $metodo( $($params)* ) $(-> $ret)? $corpo
            )*
        }

        impl ::std::ops::Deref for $nome {
            type Target = $pai;
            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }

        impl ::std::ops::DerefMut for $nome {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }
    };
}

// ============================================================================
// 8) Tempo — time(), sleep(), perf_counter(), timeit(), gmtime()/strftime()
// ============================================================================
pub mod time_utils {
    //! Equivalente (100% `std`) aos módulos `time`, `timeit` e `calendar` do Python.

    use std::fmt::Write as _;
    use std::hint::black_box;
    use std::sync::OnceLock;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    /// `time.time()` — segundos desde 1970-01-01 00:00:00 UTC como `f64`.
    #[inline]
    pub fn time() -> f64 {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs_f64(),
            Err(e) => -e.duration().as_secs_f64(),
        }
    }

    /// `time.sleep(secs)` — suspende a thread atual por `secs` segundos.
    ///
    /// # Panics
    /// Causa panic se `secs` for negativo, `NaN` ou infinito (`ValueError` no Python).
    pub fn sleep(secs: f64) {
        let d = Duration::try_from_secs_f64(secs)
            .unwrap_or_else(|_| panic!("ValueError: sleep length inválido ({secs})"));
        std::thread::sleep(d);
    }

    fn origem() -> Instant {
        static ORIGEM: OnceLock<Instant> = OnceLock::new();
        *ORIGEM.get_or_init(Instant::now)
    }

    /// `time.perf_counter()` — relógio monotônico de alta precisão em segundos (`f64`).
    #[inline]
    pub fn perf_counter() -> f64 {
        origem().elapsed().as_secs_f64()
    }

    /// `time.perf_counter_ns()` — relógio monotônico em nanossegundos (`u64`).
    #[inline]
    pub fn perf_counter_ns() -> u64 {
        origem().elapsed().as_nanos() as u64
    }

    /// `timeit.timeit(f, number=n)` — executa `f` `number` vezes protegendo o
    /// resultado com `black_box` e devolve o tempo total em segundos.
    pub fn timeit<R, F: FnMut() -> R>(number: u32, mut f: F) -> f64 {
        let inicio = Instant::now();
        for _ in 0..number {
            black_box(f());
        }
        inicio.elapsed().as_secs_f64()
    }

    /// `timeit.repeat(f, repeat=r, number=n)` — executa `r` baterias de `n` execuções.
    pub fn timeit_repeat<R, F: FnMut() -> R>(repeat: u32, number: u32, mut f: F) -> Vec<f64> {
        (0..repeat).map(|_| timeit(number, &mut f)).collect()
    }

    /// Macro ergonômica de benchmark: `timeit!(1_000, { codigo() })` ou `timeit!({ codigo() })`.
    #[macro_export]
    macro_rules! timeit {
        ($body:block) => {
            $crate::time_utils::timeit(1, || $body)
        };
        ($n:expr, $body:block) => {
            $crate::time_utils::timeit($n, || $body)
        };
    }

    /// Equivalente a `time.struct_time` em UTC (`weekday`: 0 = segunda .. 6 = domingo).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StructTime {
        pub year: i64,
        pub month: u32,
        pub day: u32,
        pub hour: u32,
        pub minute: u32,
        pub second: u32,
        pub weekday: u32,
        pub yearday: u32,
    }

    fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let mp = if m > 2 { m as i64 - 3 } else { m as i64 + 9 };
        let doy = (153 * mp + 2) / 5 + d as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn civil_from_days(z: i64) -> (i64, u32, u32) {
        let z = z + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
        (y, m, d)
    }

    /// `time.gmtime(secs)` — converte timestamp epoch em `StructTime` UTC.
    pub fn gmtime(secs: i64) -> StructTime {
        let dias = secs.div_euclid(86_400);
        let resto = secs.rem_euclid(86_400) as u32;
        let (year, month, day) = civil_from_days(dias);
        StructTime {
            year,
            month,
            day,
            hour: resto / 3_600,
            minute: resto % 3_600 / 60,
            second: resto % 60,
            weekday: (dias + 3).rem_euclid(7) as u32,
            yearday: (dias - days_from_civil(year, 1, 1) + 1) as u32,
        }
    }

    /// `gmtime_now()` — devolve a data/hora UTC atual.
    #[inline]
    pub fn gmtime_now() -> StructTime {
        gmtime(time().floor() as i64)
    }

    /// `calendar.timegm(t)` — converte `StructTime` UTC de volta para segundos epoch.
    #[inline]
    pub fn mktime_utc(t: &StructTime) -> i64 {
        days_from_civil(t.year, t.month, t.day) * 86_400
            + t.hour as i64 * 3_600
            + t.minute as i64 * 60
            + t.second as i64
    }

    const DIAS: [&str; 7] = [
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ];
    const MESES: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];

    /// `time.strftime(fmt, t)` — formata `StructTime` segundo as diretivas padrão C/Python.
    pub fn strftime(fmt: &str, t: &StructTime) -> String {
        let mut out = String::with_capacity(fmt.len() + 16);
        let mut chars = fmt.chars();
        while let Some(c) = chars.next() {
            if c != '%' {
                out.push(c);
                continue;
            }
            match chars.next() {
                Some('Y') => {
                    let _ = write!(out, "{}", t.year);
                }
                Some('y') => {
                    let _ = write!(out, "{:02}", t.year.rem_euclid(100));
                }
                Some('m') => {
                    let _ = write!(out, "{:02}", t.month);
                }
                Some('d') => {
                    let _ = write!(out, "{:02}", t.day);
                }
                Some('H') => {
                    let _ = write!(out, "{:02}", t.hour);
                }
                Some('M') => {
                    let _ = write!(out, "{:02}", t.minute);
                }
                Some('S') => {
                    let _ = write!(out, "{:02}", t.second);
                }
                Some('I') => {
                    let h = if t.hour % 12 == 0 { 12 } else { t.hour % 12 };
                    let _ = write!(out, "{:02}", h);
                }
                Some('p') => out.push_str(if t.hour < 12 { "AM" } else { "PM" }),
                Some('j') => {
                    let _ = write!(out, "{:03}", t.yearday);
                }
                Some('a') => out.push_str(&DIAS[t.weekday as usize % 7][..3]),
                Some('A') => out.push_str(DIAS[t.weekday as usize % 7]),
                Some('b') => out.push_str(&MESES[(t.month as usize + 11) % 12][..3]),
                Some('B') => out.push_str(MESES[(t.month as usize + 11) % 12]),
                Some('Z') => out.push_str("UTC"),
                Some('F') => {
                    let _ = write!(out, "{}-{:02}-{:02}", t.year, t.month, t.day);
                }
                Some('T') => {
                    let _ = write!(out, "{:02}:{:02}:{:02}", t.hour, t.minute, t.second);
                }
                Some('%') => out.push('%'),
                Some(outro) => {
                    out.push('%');
                    out.push(outro);
                }
                None => out.push('%'),
            }
        }
        out
    }
}

// ============================================================================
// 9) Rede & CyberSec Networking — socket + pwntools + ipaddress + nmap + scapy
// ============================================================================
pub mod net_utils {
    //! Wrappers de custo zero sobre `std::net` com ergonomia de `socket` do Python,
    //! expandidos na v1.5.0 com primitivas de *Exploit Development* (`pwntools`),
    //! cálculo de sub-redes CIDR (`ipaddress`), *port scanner* concorrente (`nmap`),
    //! construção de pacotes IPv4/TCP com checksum RFC 1071 (`scapy`), geração
    //! nativa de arquivos `.pcap` (Purple Team) e leitura direta de `/proc/net/*`
    //! no Linux (*Living off the Land*).

    use std::fs;
    use std::io::{self, Read, Write};
    use std::net::{self, Ipv4Addr, Shutdown, SocketAddr, SocketAddrV4, ToSocketAddrs};
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    fn para_duracao(secs: Option<f64>) -> io::Result<Option<Duration>> {
        match secs {
            None => Ok(None),
            Some(s) => Duration::try_from_secs_f64(s).map(Some).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "timeout inválido (negativo/NaN/infinito)",
                )
            }),
        }
    }

    #[derive(Debug)]
    pub struct TcpClient {
        stream: net::TcpStream,
    }

    impl TcpClient {
        pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
            Ok(TcpClient {
                stream: net::TcpStream::connect(addr)?,
            })
        }

        pub fn connect_timeout<A: ToSocketAddrs>(addr: A, secs: f64) -> io::Result<Self> {
            let dur = para_duracao(Some(secs))?.unwrap_or_default();
            let mut ultimo = io::Error::new(io::ErrorKind::NotFound, "nenhum endereço resolvido");
            for a in addr.to_socket_addrs()? {
                match net::TcpStream::connect_timeout(&a, dur) {
                    Ok(stream) => return Ok(TcpClient { stream }),
                    Err(e) => ultimo = e,
                }
            }
            Err(ultimo)
        }

        pub fn send<D: AsRef<[u8]>>(&mut self, dados: D) -> io::Result<()> {
            self.stream.write_all(dados.as_ref())
        }

        pub fn recv(&mut self, max: usize) -> io::Result<Vec<u8>> {
            let mut buf = vec![0u8; max];
            let n = self.stream.read(&mut buf)?;
            buf.truncate(n);
            Ok(buf)
        }

        pub fn recv_into(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.stream.read(buf)
        }

        pub fn recv_exact(&mut self, n: usize) -> io::Result<Vec<u8>> {
            let mut buf = vec![0u8; n];
            self.stream.read_exact(&mut buf)?;
            Ok(buf)
        }

        pub fn recv_string(&mut self, max: usize) -> io::Result<String> {
            Ok(String::from_utf8_lossy(&self.recv(max)?).into_owned())
        }

        pub fn set_timeout(&mut self, secs: Option<f64>) -> io::Result<()> {
            let d = para_duracao(secs)?;
            self.stream.set_read_timeout(d)?;
            self.stream.set_write_timeout(d)
        }

        pub fn set_nodelay(&mut self, ligado: bool) -> io::Result<()> {
            self.stream.set_nodelay(ligado)
        }

        pub fn peer_addr(&self) -> io::Result<SocketAddr> {
            self.stream.peer_addr()
        }

        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            self.stream.local_addr()
        }

        pub fn close(self) -> io::Result<()> {
            self.stream.shutdown(Shutdown::Both)
        }

        // ---------------- v1.5.0: Primitivas estilo pwntools ----------------

        /// Duplica o descritor subjacente do socket (`try_clone`), permitindo
        /// leitura e escrita simultâneas em threads separadas.
        pub fn try_clone(&self) -> io::Result<Self> {
            Ok(TcpClient {
                stream: self.stream.try_clone()?,
            })
        }

        /// Envia `dados` seguidos imediatamente de `\n` (equivalente a `p.sendline()`
        /// no `pwntools`).
        pub fn sendline<D: AsRef<[u8]>>(&mut self, dados: D) -> io::Result<()> {
            self.stream.write_all(dados.as_ref())?;
            self.stream.write_all(b"\n")?;
            self.stream.flush()
        }

        /// Lê do socket byte a byte (garantindo **zero over-read** do buffer do kernel)
        /// até encontrar a sequência `delim`, atingir `max_bytes` ou encontrar EOF.
        /// Equivalente ao `p.recvuntil(delim)` do `pwntools`.
        pub fn recvuntil<D: AsRef<[u8]>>(
            &mut self,
            delim: D,
            max_bytes: usize,
        ) -> io::Result<Vec<u8>> {
            let padrao = delim.as_ref();
            let mut acumulado = Vec::new();
            if padrao.is_empty() || max_bytes == 0 {
                return Ok(acumulado);
            }
            let mut byte = [0u8; 1];
            while acumulado.len() < max_bytes {
                match self.stream.read(&mut byte) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        acumulado.push(byte[0]);
                        if acumulado.len() >= padrao.len()
                            && &acumulado[acumulado.len() - padrao.len()..] == padrao
                        {
                            break;
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
            Ok(acumulado)
        }

        /// Lê uma linha completa até `\n` (ou até `max_bytes`) sem *over-read*.
        /// Equivalente a `p.recvline()` no `pwntools`.
        #[inline]
        pub fn recvline(&mut self, max_bytes: usize) -> io::Result<Vec<u8>> {
            self.recvuntil(b"\n", max_bytes)
        }

        /// Lê uma linha até `\n` como `String` (UTF-8 lossy) e remove `\r\n` ou `\n`
        /// do final. Ideal para capturar prompts, banners e flags.
        pub fn recvline_string(&mut self, max_bytes: usize) -> io::Result<String> {
            let bytes = self.recvline(max_bytes)?;
            let mut s = String::from_utf8_lossy(&bytes).into_owned();
            if s.ends_with('\n') {
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
            }
            Ok(s)
        }

        /// Aguarda `delim` via [`TcpClient::recvuntil`] e envia `payload` logo em seguida.
        /// Devolve os bytes recebidos até o delimitador. Equivalente a `p.sendafter()`.
        pub fn sendafter<D: AsRef<[u8]>, P: AsRef<[u8]>>(
            &mut self,
            delim: D,
            payload: P,
            max_bytes: usize,
        ) -> io::Result<Vec<u8>> {
            let recebido = self.recvuntil(delim, max_bytes)?;
            self.send(payload)?;
            Ok(recebido)
        }

        /// Aguarda `delim` via [`TcpClient::recvuntil`] e envia `payload` seguido de `\n`.
        /// Devolve os bytes recebidos até o delimitador. Equivalente a `p.sendlineafter()`.
        pub fn sendlineafter<D: AsRef<[u8]>, P: AsRef<[u8]>>(
            &mut self,
            delim: D,
            payload: P,
            max_bytes: usize,
        ) -> io::Result<Vec<u8>> {
            let recebido = self.recvuntil(delim, max_bytes)?;
            self.sendline(payload)?;
            Ok(recebido)
        }

        /// Conecta o socket interativamente ao terminal (`stdin` ↔ `socket` ↔ `stdout`),
        /// equivalente ao `p.interactive()` do `pwntools` para interação com shells remotos.
        pub fn interactive(&mut self) -> io::Result<()> {
            let mut leitor = self.stream.try_clone()?;
            let mut escritor = self.stream.try_clone()?;

            std::thread::scope(|s| {
                let t_out = s.spawn(move || -> io::Result<()> {
                    let mut buf = [0u8; 1024];
                    let mut stdout = io::stdout();
                    loop {
                        match leitor.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                stdout.write_all(&buf[..n])?;
                                stdout.flush()?;
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                            Err(_) => break,
                        }
                    }
                    Ok(())
                });

                let mut stdin = io::stdin();
                let mut buf_in = [0u8; 1024];
                loop {
                    match stdin.read(&mut buf_in) {
                        Ok(0) => {
                            let _ = escritor.shutdown(Shutdown::Write);
                            break;
                        }
                        Ok(n) => {
                            if escritor.write_all(&buf_in[..n]).is_err() {
                                break;
                            }
                            let _ = escritor.flush();
                        }
                        Err(_) => break,
                    }
                }
                let _ = t_out.join();
            });
            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct TcpServer {
        listener: net::TcpListener,
    }

    impl TcpServer {
        pub fn listen<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
            Ok(TcpServer {
                listener: net::TcpListener::bind(addr)?,
            })
        }

        pub fn accept(&self) -> io::Result<(TcpClient, SocketAddr)> {
            let (stream, addr) = self.listener.accept()?;
            Ok((TcpClient { stream }, addr))
        }

        pub fn incoming(&self) -> impl Iterator<Item = io::Result<TcpClient>> + '_ {
            self.listener
                .incoming()
                .map(|r| r.map(|stream| TcpClient { stream }))
        }

        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            self.listener.local_addr()
        }
    }

    #[derive(Debug)]
    pub struct UdpSock {
        socket: net::UdpSocket,
    }

    impl UdpSock {
        pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
            Ok(UdpSock {
                socket: net::UdpSocket::bind(addr)?,
            })
        }

        pub fn send_to<D: AsRef<[u8]>, A: ToSocketAddrs>(
            &self,
            dados: D,
            addr: A,
        ) -> io::Result<usize> {
            self.socket.send_to(dados.as_ref(), addr)
        }

        pub fn recv_from(&self, max: usize) -> io::Result<(Vec<u8>, SocketAddr)> {
            let mut buf = vec![0u8; max];
            let (n, origem) = self.socket.recv_from(&mut buf)?;
            buf.truncate(n);
            Ok((buf, origem))
        }

        pub fn recv_from_into(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
            self.socket.recv_from(buf)
        }

        pub fn set_timeout(&self, secs: Option<f64>) -> io::Result<()> {
            let d = para_duracao(secs)?;
            self.socket.set_read_timeout(d)?;
            self.socket.set_write_timeout(d)
        }

        pub fn local_addr(&self) -> io::Result<SocketAddr> {
            self.socket.local_addr()
        }
    }

    pub fn getaddrinfo(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
        Ok((host, port).to_socket_addrs()?.collect())
    }

    pub fn gethostbyname(host: &str) -> io::Result<Ipv4Addr> {
        (host, 0)
            .to_socket_addrs()?
            .find_map(|a| match a {
                SocketAddr::V4(v4) => Some(*v4.ip()),
                SocketAddr::V6(_) => None,
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("nenhum IPv4 para '{host}'"),
                )
            })
    }

    // ========================================================================
    // v1.5.0: Parser de CIDR IPv4 (ipaddress) e Port Ranges estilo Nmap
    // ========================================================================

    /// Representa um bloco CIDR IPv4 (ex.: `192.168.1.0/24`) e funciona como
    /// iterador preguiçoso ($O(1)$ em memória) sobre todos os endereços do bloco.
    /// Equivalente a `ipaddress.IPv4Network` do Python.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Ipv4Cidr {
        base_ip: u32,
        prefix: u8,
        atual: u64,
        fim_exclusivo: u64,
    }

    impl Ipv4Cidr {
        /// Interpreta uma notação `"A.B.C.D/PREFIXO"` (ou `"A.B.C.D"`, assumindo `/32`).
        pub fn parse(cidr: &str) -> Result<Self, String> {
            let s = cidr.trim();
            let (ip_str, pref_str) = match s.split_once('/') {
                Some((i, p)) => (i.trim(), p.trim()),
                None => (s, "32"),
            };
            let ip: Ipv4Addr = ip_str
                .parse()
                .map_err(|_| format!("ValueError: IPv4 inválido em CIDR '{cidr}'"))?;
            let prefix: u8 = pref_str
                .parse()
                .map_err(|_| format!("ValueError: prefixo CIDR inválido em '{cidr}'"))?;
            if prefix > 32 {
                return Err(format!(
                    "ValueError: prefixo CIDR fora do intervalo 0..=32 ({prefix})"
                ));
            }
            let mask = if prefix == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix)
            };
            let base_ip = u32::from(ip) & mask;
            let qtd = 1u64 << (32 - prefix as u64);
            Ok(Ipv4Cidr {
                base_ip,
                prefix,
                atual: base_ip as u64,
                fim_exclusivo: (base_ip as u64) + qtd,
            })
        }

        /// Devolve o comprimento do prefixo (`0..=32`).
        #[inline]
        pub fn prefix_len(&self) -> u8 {
            self.prefix
        }

        /// Devolve a máscara de sub-rede (ex.: `255.255.255.0` para `/24`).
        #[inline]
        pub fn netmask(&self) -> Ipv4Addr {
            let mask = if self.prefix == 0 {
                0u32
            } else {
                !0u32 << (32 - self.prefix)
            };
            Ipv4Addr::from(mask)
        }

        /// Devolve o endereço de rede do bloco.
        #[inline]
        pub fn network(&self) -> Ipv4Addr {
            Ipv4Addr::from(self.base_ip)
        }

        /// Devolve o endereço de broadcast do bloco.
        #[inline]
        pub fn broadcast(&self) -> Ipv4Addr {
            let host_mask = if self.prefix == 32 {
                0u32
            } else if self.prefix == 0 {
                !0u32
            } else {
                (1u32 << (32 - self.prefix)) - 1
            };
            Ipv4Addr::from(self.base_ip | host_mask)
        }

        /// Verifica em $O(1)$ se `ip` pertence a esta sub-rede.
        #[inline]
        pub fn contains(&self, ip: Ipv4Addr) -> bool {
            let mask = u32::from(self.netmask());
            (u32::from(ip) & mask) == self.base_ip
        }

        /// Total de endereços IPv4 abrangidos pelo bloco CIDR.
        #[inline]
        pub fn total_addresses(&self) -> u64 {
            1u64 << (32 - self.prefix as u64)
        }

        /// Devolve um iterador apenas sobre os hosts utilizáveis (exclui endereço de
        /// rede e broadcast quando `prefix <= 30`, igual a `net.hosts()` do Python).
        #[inline]
        pub fn hosts(&self) -> Self {
            if self.prefix <= 30 {
                Ipv4Cidr {
                    base_ip: self.base_ip,
                    prefix: self.prefix,
                    atual: (self.base_ip as u64) + 1,
                    fim_exclusivo: self.fim_exclusivo.saturating_sub(1),
                }
            } else {
                *self
            }
        }
    }

    impl Iterator for Ipv4Cidr {
        type Item = Ipv4Addr;

        #[inline]
        fn next(&mut self) -> Option<Ipv4Addr> {
            if self.atual >= self.fim_exclusivo {
                return None;
            }
            let ip = Ipv4Addr::from(self.atual as u32);
            self.atual += 1;
            Some(ip)
        }

        #[inline]
        fn size_hint(&self) -> (usize, Option<usize>) {
            let restante = self.fim_exclusivo.saturating_sub(self.atual) as usize;
            (restante, Some(restante))
        }
    }

    /// Função livre ergonômica para criar um [`Ipv4Cidr`] a partir de `"A.B.C.D/N"`.
    #[inline]
    pub fn parse_cidr(cidr: &str) -> Result<Ipv4Cidr, String> {
        Ipv4Cidr::parse(cidr)
    }

    /// Interpreta uma especificação de portas estilo Nmap (ex.: `"22,80,443,8000-8005"`),
    /// devolvendo um `Vec<u16>` ordenado e sem duplicatas.
    pub fn parse_ports(spec: &str) -> Result<Vec<u16>, String> {
        let mut portas = Vec::new();
        for parte in spec.split(',') {
            let item = parte.trim();
            if item.is_empty() {
                continue;
            }
            if let Some((ini_s, fim_s)) = item.split_once('-') {
                let ini: u16 = ini_s
                    .trim()
                    .parse()
                    .map_err(|_| format!("ValueError: porta inicial inválida em '{item}'"))?;
                let fim: u16 = fim_s
                    .trim()
                    .parse()
                    .map_err(|_| format!("ValueError: porta final inválida em '{item}'"))?;
                if ini == 0 || fim == 0 || ini > fim {
                    return Err(format!(
                        "ValueError: intervalo de portas inválido '{item}' (use 1..=65535 com ini <= fim)"
                    ));
                }
                for p in ini..=fim {
                    portas.push(p);
                }
            } else {
                let p: u16 = item
                    .parse()
                    .map_err(|_| format!("ValueError: porta inválida '{item}'"))?;
                if p == 0 {
                    return Err("ValueError: porta 0 não é válida em parse_ports".to_string());
                }
                portas.push(p);
            }
        }
        portas.sort_unstable();
        portas.dedup();
        Ok(portas)
    }

    // ========================================================================
    // v1.5.0: Port Scanner & Banner Grabber Concorrente (std::thread::scope)
    // ========================================================================

    /// Resultado individual de sondagem de porta TCP (`tcp_probe` / `tcp_scan`).
    #[derive(Debug, Clone, PartialEq)]
    pub struct PortScanResult {
        pub host: String,
        pub port: u16,
        pub open: bool,
        pub banner: Option<String>,
        pub latency_ms: f64,
    }

    /// Testa uma única porta TCP com timeout em segundos, mede o tempo de resposta
    /// (`latency_ms`) e, se `grab_banner == true`, tenta capturar o banner inicial
    /// do serviço (limpando quebras de linha finais).
    pub fn tcp_probe(
        host: &str,
        port: u16,
        timeout_secs: f64,
        grab_banner: bool,
    ) -> PortScanResult {
        let t0 = Instant::now();
        match TcpClient::connect_timeout((host, port), timeout_secs) {
            Ok(mut cli) => {
                let latency_ms = t0.elapsed().as_secs_f64() * 1000.0;
                let mut banner = None;
                if grab_banner {
                    let _ = cli.set_timeout(Some(timeout_secs));
                    if let Ok(dados) = cli.recv(256) {
                        if !dados.is_empty() {
                            let texto = String::from_utf8_lossy(&dados).trim().to_string();
                            if !texto.is_empty() {
                                banner = Some(texto);
                            }
                        }
                    }
                }
                let _ = cli.close();
                PortScanResult {
                    host: host.to_string(),
                    port,
                    open: true,
                    banner,
                    latency_ms,
                }
            }
            Err(_) => PortScanResult {
                host: host.to_string(),
                port,
                open: false,
                banner: None,
                latency_ms: t0.elapsed().as_secs_f64() * 1000.0,
            },
        }
    }

    /// Executa um scan TCP concorrente sobre `ports` usando `std::thread::scope`
    /// com até `threads` workers em paralelo. Devolve os resultados ordenados por porta.
    pub fn tcp_scan(
        host: &str,
        ports: &[u16],
        timeout_secs: f64,
        threads: usize,
        grab_banner: bool,
    ) -> Vec<PortScanResult> {
        if ports.is_empty() {
            return Vec::new();
        }
        let num_workers = threads.max(1).min(ports.len());
        let indice = AtomicUsize::new(0);
        let resultados: Mutex<Vec<PortScanResult>> = Mutex::new(Vec::with_capacity(ports.len()));

        std::thread::scope(|s| {
            for _ in 0..num_workers {
                s.spawn(|| {
                    let mut lote_local = Vec::new();
                    loop {
                        let idx = indice.fetch_add(1, Ordering::Relaxed);
                        if idx >= ports.len() {
                            break;
                        }
                        let porta = ports[idx];
                        lote_local.push(tcp_probe(host, porta, timeout_secs, grab_banner));
                    }
                    if !lote_local.is_empty() {
                        if let Ok(mut guard) = resultados.lock() {
                            guard.extend(lote_local);
                        }
                    }
                });
            }
        });

        let mut final_vec = resultados.into_inner().unwrap_or_default();
        final_vec.sort_by_key(|r| r.port);
        final_vec
    }

    // ========================================================================
    // v1.5.0: Packet Crafting IPv4/TCP (RFC 1071) & Gerador Nativo de .pcap
    // ========================================================================

    pub const TCP_FIN: u8 = 0x01;
    pub const TCP_SYN: u8 = 0x02;
    pub const TCP_RST: u8 = 0x04;
    pub const TCP_PSH: u8 = 0x08;
    pub const TCP_ACK: u8 = 0x10;
    pub const TCP_URG: u8 = 0x20;
    pub const TCP_ECE: u8 = 0x40;
    pub const TCP_CWR: u8 = 0x80;

    /// Calcula o Internet Checksum padrão (RFC 1071) em complemento de um de 16 bits.
    pub fn rfc1071_checksum(dados: &[u8]) -> u16 {
        let mut soma: u32 = 0;
        let mut chunks = dados.chunks_exact(2);
        for par in &mut chunks {
            soma += u16::from_be_bytes([par[0], par[1]]) as u32;
        }
        if let Some(&ultimo) = chunks.remainder().first() {
            soma += u16::from_be_bytes([ultimo, 0]) as u32;
        }
        while (soma >> 16) != 0 {
            soma = (soma & 0xFFFF) + (soma >> 16);
        }
        !(soma as u16)
    }

    /// Calcula o checksum de um segmento TCP incluindo o pseudo-cabeçalho IPv4.
    pub fn ipv4_tcp_checksum(src_ip: Ipv4Addr, dst_ip: Ipv4Addr, tcp_segment: &[u8]) -> u16 {
        let mut buf = Vec::with_capacity(12 + tcp_segment.len());
        buf.extend_from_slice(&src_ip.octets());
        buf.extend_from_slice(&dst_ip.octets());
        buf.push(0);
        buf.push(6); // Protocolo 6 = TCP
        buf.extend_from_slice(&(tcp_segment.len() as u16).to_be_bytes());
        buf.extend_from_slice(tcp_segment);
        rfc1071_checksum(&buf)
    }

    /// Construtor e dissecador de pacotes IPv4/TCP estilo Scapy (`IP()/TCP()/Raw()`).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Ipv4TcpPacket {
        pub src_ip: Ipv4Addr,
        pub dst_ip: Ipv4Addr,
        pub src_port: u16,
        pub dst_port: u16,
        pub seq: u32,
        pub ack: u32,
        pub flags: u8,
        pub window: u16,
        pub ttl: u8,
        pub identification: u16,
        pub payload: Vec<u8>,
    }

    impl Ipv4TcpPacket {
        /// Cria um pacote IPv4/TCP base (TTL=64, Window=64240, Seq=1, Ack=0).
        pub fn new(
            src_ip: Ipv4Addr,
            dst_ip: Ipv4Addr,
            src_port: u16,
            dst_port: u16,
            flags: u8,
        ) -> Self {
            Ipv4TcpPacket {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                seq: 1,
                ack: 0,
                flags,
                window: 64240,
                ttl: 64,
                identification: 0x1337,
                payload: Vec::new(),
            }
        }

        /// Define a carga útil (payload) do segmento TCP.
        pub fn with_payload<D: AsRef<[u8]>>(mut self, payload: D) -> Self {
            self.payload = payload.as_ref().to_vec();
            self
        }

        /// Define o número de sequência (`seq`).
        pub fn with_seq(mut self, seq: u32) -> Self {
            self.seq = seq;
            self
        }

        /// Define o número de confirmação (`ack`).
        pub fn with_ack(mut self, ack: u32) -> Self {
            self.ack = ack;
            self
        }

        /// Define o Time-To-Live (`ttl`) do cabeçalho IPv4.
        pub fn with_ttl(mut self, ttl: u8) -> Self {
            self.ttl = ttl;
            self
        }

        /// Serializa o pacote completo (20 bytes IPv4 + 20 bytes TCP + payload)
        /// preenchendo automaticamente os checksums IPv4 e TCP.
        pub fn build(&self) -> Vec<u8> {
            let total_len = (40 + self.payload.len()) as u16;
            let mut tcp = Vec::with_capacity(20 + self.payload.len());
            tcp.extend_from_slice(&self.src_port.to_be_bytes());
            tcp.extend_from_slice(&self.dst_port.to_be_bytes());
            tcp.extend_from_slice(&self.seq.to_be_bytes());
            tcp.extend_from_slice(&self.ack.to_be_bytes());
            tcp.push(0x50); // Data Offset = 5 (20 bytes)
            tcp.push(self.flags);
            tcp.extend_from_slice(&self.window.to_be_bytes());
            tcp.extend_from_slice(&0u16.to_be_bytes()); // Checksum provisório
            tcp.extend_from_slice(&0u16.to_be_bytes()); // Urgent Pointer
            tcp.extend_from_slice(&self.payload);

            let tcp_csum = ipv4_tcp_checksum(self.src_ip, self.dst_ip, &tcp);
            tcp[16..18].copy_from_slice(&tcp_csum.to_be_bytes());

            let mut ipv4 = Vec::with_capacity(total_len as usize);
            ipv4.push(0x45); // Version 4, IHL 5 (20 bytes)
            ipv4.push(0x00); // DSCP / ECN
            ipv4.extend_from_slice(&total_len.to_be_bytes());
            ipv4.extend_from_slice(&self.identification.to_be_bytes());
            ipv4.extend_from_slice(&0x4000u16.to_be_bytes()); // Don't Fragment (DF)
            ipv4.push(self.ttl);
            ipv4.push(6); // Protocol 6 (TCP)
            ipv4.extend_from_slice(&0u16.to_be_bytes()); // Header checksum provisório
            ipv4.extend_from_slice(&self.src_ip.octets());
            ipv4.extend_from_slice(&self.dst_ip.octets());

            let ip_csum = rfc1071_checksum(&ipv4);
            ipv4[10..12].copy_from_slice(&ip_csum.to_be_bytes());
            ipv4.extend_from_slice(&tcp);
            ipv4
        }

        /// Disseca um pacote IPv4/TCP bruto em uma struct [`Ipv4TcpPacket`].
        pub fn parse(raw: &[u8]) -> Option<Self> {
            if raw.len() < 40 || (raw[0] >> 4) != 4 {
                return None;
            }
            let ihl = ((raw[0] & 0x0F) as usize) * 4;
            if ihl < 20 || raw.len() < ihl + 20 || raw[9] != 6 {
                return None;
            }
            let identification = u16::from_be_bytes([raw[4], raw[5]]);
            let ttl = raw[8];
            let src_ip = Ipv4Addr::new(raw[12], raw[13], raw[14], raw[15]);
            let dst_ip = Ipv4Addr::new(raw[16], raw[17], raw[18], raw[19]);

            let tcp = &raw[ihl..];
            let src_port = u16::from_be_bytes([tcp[0], tcp[1]]);
            let dst_port = u16::from_be_bytes([tcp[2], tcp[3]]);
            let seq = u32::from_be_bytes([tcp[4], tcp[5], tcp[6], tcp[7]]);
            let ack = u32::from_be_bytes([tcp[8], tcp[9], tcp[10], tcp[11]]);
            let data_offset = ((tcp[12] >> 4) as usize) * 4;
            if data_offset < 20 || tcp.len() < data_offset {
                return None;
            }
            let flags = tcp[13];
            let window = u16::from_be_bytes([tcp[14], tcp[15]]);
            let payload = tcp[data_offset..].to_vec();

            Some(Ipv4TcpPacket {
                src_ip,
                dst_ip,
                src_port,
                dst_port,
                seq,
                ack,
                flags,
                window,
                ttl,
                identification,
                payload,
            })
        }
    }

    /// Gerador nativo de arquivos `.pcap` (formato padrão Libpcap v2.4) sem
    /// depender de `libpcap`. Ideal em operações Purple Team para gerar tráfego
    /// sintético e validar regras de detecção no Suricata, Snort, Zeek ou Wireshark.
    #[derive(Debug, Clone)]
    pub struct PcapWriter {
        buf: Vec<u8>,
        linktype: u32,
    }

    impl PcapWriter {
        /// Inicializa um `.pcap` com `LINKTYPE_RAW` (`101`), onde cada pacote começa
        /// diretamente no cabeçalho IPv4/IPv6 (sem cabeçalho Ethernet).
        pub fn new_raw_ip() -> Self {
            Self::with_linktype(101)
        }

        /// Inicializa um `.pcap` com `LINKTYPE_ETHERNET` (`1`), onde cada quadro
        /// possui um cabeçalho Ethernet de 14 bytes antes do pacote IPv4.
        pub fn new_ethernet() -> Self {
            Self::with_linktype(1)
        }

        fn with_linktype(linktype: u32) -> Self {
            let mut buf = Vec::with_capacity(256);
            buf.extend_from_slice(&0xA1B2_C3D4u32.to_le_bytes()); // Magic number (microssegundos)
            buf.extend_from_slice(&2u16.to_le_bytes()); // Major version 2
            buf.extend_from_slice(&4u16.to_le_bytes()); // Minor version 4
            buf.extend_from_slice(&0i32.to_le_bytes()); // thiszone (UTC)
            buf.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
            buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
            buf.extend_from_slice(&linktype.to_le_bytes()); // network linktype
            PcapWriter { buf, linktype }
        }

        /// Adiciona um pacote bruto com timestamp explícito (`ts_sec`, `ts_usec`).
        pub fn add_packet(&mut self, ts_sec: u32, ts_usec: u32, dados: &[u8]) {
            let tam = dados.len() as u32;
            self.buf.extend_from_slice(&ts_sec.to_le_bytes());
            self.buf.extend_from_slice(&ts_usec.to_le_bytes());
            self.buf.extend_from_slice(&tam.to_le_bytes()); // incl_len
            self.buf.extend_from_slice(&tam.to_le_bytes()); // orig_len
            self.buf.extend_from_slice(dados);
        }

        /// Adiciona um pacote bruto usando o timestamp atual do sistema.
        pub fn add_packet_now(&mut self, dados: &[u8]) {
            let d = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            self.add_packet(d.as_secs() as u32, d.subsec_micros(), dados);
        }

        /// Encapsula um pacote IPv4 em um quadro Ethernet (`EtherType = 0x0800`) e o
        /// grava no buffer do PCAP.
        pub fn add_ipv4_as_ethernet(
            &mut self,
            ts_sec: u32,
            ts_usec: u32,
            src_mac: [u8; 6],
            dst_mac: [u8; 6],
            ipv4_packet: &[u8],
        ) {
            let mut frame = Vec::with_capacity(14 + ipv4_packet.len());
            frame.extend_from_slice(&dst_mac);
            frame.extend_from_slice(&src_mac);
            frame.extend_from_slice(&0x0800u16.to_be_bytes());
            frame.extend_from_slice(ipv4_packet);
            self.add_packet(ts_sec, ts_usec, &frame);
        }

        /// Devolve uma fatia com todos os bytes do arquivo `.pcap` gerado.
        #[inline]
        pub fn as_bytes(&self) -> &[u8] {
            &self.buf
        }

        /// Consome o escritor e devolve o `Vec<u8>` do arquivo `.pcap`.
        #[inline]
        pub fn into_bytes(self) -> Vec<u8> {
            self.buf
        }

        /// Salva o conteúdo `.pcap` diretamente em disco.
        pub fn save<P: AsRef<Path>>(&self, caminho: P) -> io::Result<()> {
            fs::write(caminho, &self.buf)
        }
    }

    // ========================================================================
    // v1.5.0: Living off the Land — Leitura direta de /proc/net/tcp e /proc/net/arp
    // ========================================================================

    /// Entrada da tabela de conexões TCP do kernel Linux (`/proc/net/tcp`).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProcTcpEntry {
        pub sl: usize,
        pub local_addr: SocketAddrV4,
        pub rem_addr: SocketAddrV4,
        pub state: &'static str,
        pub uid: u32,
        pub inode: u64,
    }

    fn estado_tcp_linux(codigo: u8) -> &'static str {
        match codigo {
            0x01 => "ESTABLISHED",
            0x02 => "SYN_SENT",
            0x03 => "SYN_RECV",
            0x04 => "FIN_WAIT1",
            0x05 => "FIN_WAIT2",
            0x06 => "TIME_WAIT",
            0x07 => "CLOSE",
            0x08 => "CLOSE_WAIT",
            0x09 => "LAST_ACK",
            0x0A => "LISTEN",
            0x0B => "CLOSING",
            0x0C => "NEW_SYN_RECV",
            _ => "UNKNOWN",
        }
    }

    fn parse_hex_ip_port_le(campo: &str) -> Option<SocketAddrV4> {
        let (ip_hex, port_hex) = campo.split_once(':')?;
        let ip_le = u32::from_str_radix(ip_hex, 16).ok()?;
        let port = u16::from_str_radix(port_hex, 16).ok()?;
        // Em sistemas little-endian (x86_64/ARM), /proc/net/tcp armazena os 4 octetos
        // invertidos na representação hexadecimal de 32 bits.
        let octetos = ip_le.to_le_bytes();
        Some(SocketAddrV4::new(
            Ipv4Addr::new(octetos[0], octetos[1], octetos[2], octetos[3]),
            port,
        ))
    }

    /// Faz o parse do conteúdo textual de `/proc/net/tcp` em structs [`ProcTcpEntry`].
    pub fn parse_proc_net_tcp_str(conteudo: &str) -> Vec<ProcTcpEntry> {
        let mut entradas = Vec::new();
        for (i, linha) in conteudo.lines().enumerate() {
            if i == 0 {
                continue; // Pula cabeçalho "sl  local_address rem_address ..."
            }
            let cols: Vec<&str> = linha.split_whitespace().collect();
            if cols.len() < 10 {
                continue;
            }
            let sl = cols[0].trim_end_matches(':').parse::<usize>().unwrap_or(0);
            let local_addr = match parse_hex_ip_port_le(cols[1]) {
                Some(a) => a,
                None => continue,
            };
            let rem_addr = match parse_hex_ip_port_le(cols[2]) {
                Some(a) => a,
                None => continue,
            };
            let state_code = u8::from_str_radix(cols[3], 16).unwrap_or(0);
            let uid = cols[7].parse::<u32>().unwrap_or(0);
            let inode = cols[9].parse::<u64>().unwrap_or(0);

            entradas.push(ProcTcpEntry {
                sl,
                local_addr,
                rem_addr,
                state: estado_tcp_linux(state_code),
                uid,
                inode,
            });
        }
        entradas
    }

    /// Lê diretamente `/proc/net/tcp` no Linux sem invocar `ss` ou `netstat`
    /// (*Living off the Land*).
    pub fn proc_net_tcp() -> io::Result<Vec<ProcTcpEntry>> {
        let conteudo = fs::read_to_string("/proc/net/tcp")?;
        Ok(parse_proc_net_tcp_str(&conteudo))
    }

    /// Entrada da tabela ARP do kernel Linux (`/proc/net/arp`).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ProcArpEntry {
        pub ip: Ipv4Addr,
        pub hw_type: String,
        pub flags: String,
        pub mac: String,
        pub mask: String,
        pub device: String,
    }

    /// Faz o parse do conteúdo textual de `/proc/net/arp` em structs [`ProcArpEntry`].
    pub fn parse_proc_net_arp_str(conteudo: &str) -> Vec<ProcArpEntry> {
        let mut entradas = Vec::new();
        for (i, linha) in conteudo.lines().enumerate() {
            if i == 0 {
                continue; // Pula cabeçalho "IP address       HW type ..."
            }
            let cols: Vec<&str> = linha.split_whitespace().collect();
            if cols.len() < 6 {
                continue;
            }
            if let Ok(ip) = cols[0].parse::<Ipv4Addr>() {
                entradas.push(ProcArpEntry {
                    ip,
                    hw_type: cols[1].to_string(),
                    flags: cols[2].to_string(),
                    mac: cols[3].to_string(),
                    mask: cols[4].to_string(),
                    device: cols[5].to_string(),
                });
            }
        }
        entradas
    }

    /// Lê diretamente `/proc/net/arp` no Linux para descoberta passiva de hosts
    /// na rede local sem emitir pacotes ARP.
    pub fn proc_net_arp() -> io::Result<Vec<ProcArpEntry>> {
        let conteudo = fs::read_to_string("/proc/net/arp")?;
        Ok(parse_proc_net_arp_str(&conteudo))
    }
}

// ============================================================================
// 10) Sistema de arquivos — os / os.path / pathlib / shutil + I/O de texto
// ============================================================================
pub mod fs_utils {
    //! Operações de sistema de arquivos e leitura/escrita rápida de arquivos
    //! equivalentes a `os`, `os.path`, `shutil` e `open()` do Python.

    use std::fs::{self, OpenOptions};
    use std::io::{self, Write};
    #[cfg(unix)]
    use std::os::unix::fs::{symlink as unix_symlink, MetadataExt, PermissionsExt};
    use std::path::{Component, Path, PathBuf};

    // ---------------- Leitura e escrita rápida (open().read() / write()) ----

    /// Lê todo o conteúdo de um arquivo de texto para uma `String`
    /// (equivalente a `open(caminho).read()` no Python).
    #[inline]
    pub fn read_text<P: AsRef<Path>>(caminho: P) -> io::Result<String> {
        fs::read_to_string(caminho)
    }

    /// Lê todas as linhas de um arquivo de texto sem os terminadores `\n`/`\r\n`
    /// (equivalente a `[line.rstrip('\r\n') for line in open(caminho)]`).
    pub fn readlines<P: AsRef<Path>>(caminho: P) -> io::Result<Vec<String>> {
        Ok(fs::read_to_string(caminho)?
            .lines()
            .map(String::from)
            .collect())
    }

    /// Escreve (sobrescreve ou cria) texto em um arquivo
    /// (equivalente a `open(caminho, 'w').write(conteudo)`).
    #[inline]
    pub fn write_text<P: AsRef<Path>, C: AsRef<[u8]>>(caminho: P, conteudo: C) -> io::Result<()> {
        fs::write(caminho, conteudo)
    }

    /// Acrescenta texto ao final de um arquivo, criando-o se não existir
    /// (equivalente a `open(caminho, 'a').write(conteudo)`).
    pub fn append_text<P: AsRef<Path>, C: AsRef<[u8]>>(caminho: P, conteudo: C) -> io::Result<()> {
        let mut arq = OpenOptions::new().create(true).append(true).open(caminho)?;
        arq.write_all(conteudo.as_ref())
    }

    // ---------------- Consultas (os.path.exists/isfile/isdir/islink) -------
    #[inline]
    pub fn exists<P: AsRef<Path>>(caminho: P) -> bool {
        caminho.as_ref().exists()
    }
    #[inline]
    pub fn isfile<P: AsRef<Path>>(caminho: P) -> bool {
        caminho.as_ref().is_file()
    }
    #[inline]
    pub fn isdir<P: AsRef<Path>>(caminho: P) -> bool {
        caminho.as_ref().is_dir()
    }
    #[inline]
    pub fn islink<P: AsRef<Path>>(caminho: P) -> bool {
        caminho.as_ref().is_symlink()
    }

    // ---------------- Listar / criar / apagar (os / shutil) ----------------
    pub fn listdir<P: AsRef<Path>>(caminho: P) -> io::Result<Vec<String>> {
        fs::read_dir(caminho)?
            .map(|entrada| Ok(entrada?.file_name().to_string_lossy().into_owned()))
            .collect()
    }

    pub fn mkdir<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        fs::create_dir(caminho)
    }

    pub fn makedirs<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        fs::create_dir_all(caminho)
    }

    pub fn remove<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        fs::remove_file(caminho)
    }

    pub fn rmdir<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        fs::remove_dir(caminho)
    }

    pub fn rmtree<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        fs::remove_dir_all(caminho)
    }

    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(de: P, para: Q) -> io::Result<()> {
        fs::rename(de, para)
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(de: P, para: Q) -> io::Result<u64> {
        fs::copy(de, para)
    }

    // ---------------- Diretório atual (os.getcwd/chdir) --------------------
    pub fn getcwd() -> io::Result<String> {
        Ok(std::env::current_dir()?.to_string_lossy().into_owned())
    }

    pub fn chdir<P: AsRef<Path>>(caminho: P) -> io::Result<()> {
        std::env::set_current_dir(caminho)
    }

    // ---------------- Manipulação de caminhos (os.path) --------------------
    pub fn join_path(partes: &[&str]) -> String {
        let mut base = PathBuf::new();
        for p in partes {
            base.push(p);
        }
        base.to_string_lossy().into_owned()
    }

    pub fn basename<P: AsRef<Path>>(caminho: P) -> String {
        caminho
            .as_ref()
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn dirname<P: AsRef<Path>>(caminho: P) -> String {
        caminho
            .as_ref()
            .parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn splitext<P: AsRef<Path>>(caminho: P) -> (String, String) {
        let p = caminho.as_ref();
        let stem = p
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ext = p
            .extension()
            .map(|s| format!(".{}", s.to_string_lossy()))
            .unwrap_or_default();
        let raiz = match p.parent().filter(|d| !d.as_os_str().is_empty()) {
            Some(dir) => dir.join(&stem).to_string_lossy().into_owned(),
            None => stem,
        };
        (raiz, ext)
    }

    pub fn abspath<P: AsRef<Path>>(caminho: P) -> io::Result<String> {
        let caminho = caminho.as_ref();
        let base = if caminho.is_absolute() {
            caminho.to_path_buf()
        } else {
            std::env::current_dir()?.join(caminho)
        };
        let mut normalizado = PathBuf::new();
        for componente in base.components() {
            match componente {
                Component::ParentDir => {
                    normalizado.pop();
                }
                Component::CurDir => {}
                outro => normalizado.push(outro.as_os_str()),
            }
        }
        Ok(normalizado.to_string_lossy().into_owned())
    }

    pub fn realpath<P: AsRef<Path>>(caminho: P) -> io::Result<String> {
        Ok(fs::canonicalize(caminho)?.to_string_lossy().into_owned())
    }

    pub fn expanduser(caminho: &str) -> String {
        if let Some(resto) = caminho.strip_prefix('~') {
            if resto.is_empty() || resto.starts_with('/') {
                if let Ok(home) = std::env::var("HOME") {
                    return format!("{home}{resto}");
                }
            }
        }
        caminho.to_string()
    }

    // ---------------- Percurso recursivo seguro (os.walk) ------------------
    pub type WalkItem = (String, Vec<String>, Vec<String>);

    pub struct Walk {
        pilha: Vec<PathBuf>,
    }

    pub fn walk<P: AsRef<Path>>(raiz: P) -> Walk {
        Walk {
            pilha: vec![raiz.as_ref().to_path_buf()],
        }
    }

    impl Iterator for Walk {
        type Item = io::Result<WalkItem>;

        fn next(&mut self) -> Option<Self::Item> {
            let dir = self.pilha.pop()?;
            let entradas = match fs::read_dir(&dir) {
                Ok(e) => e,
                Err(erro) => return Some(Err(erro)),
            };

            let mut subdirs = Vec::new();
            let mut arquivos = Vec::new();
            for entrada in entradas {
                let entrada = match entrada {
                    Ok(e) => e,
                    Err(erro) => return Some(Err(erro)),
                };
                let nome = entrada.file_name().to_string_lossy().into_owned();
                // Usa file_type() em vez de path().is_dir(): não segue links
                // simbólicos (impede loop infinito) e evita syscall stat() extra!
                let eh_dir = entrada.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                if eh_dir {
                    self.pilha.push(entrada.path());
                    subdirs.push(nome);
                } else {
                    arquivos.push(nome);
                }
            }

            Some(Ok((
                dir.to_string_lossy().into_owned(),
                subdirs,
                arquivos,
            )))
        }
    }

    // ---------------- Glob mínimo (* e ?) ----------------------------------
    fn glob_casa(padrao: &[char], nome: &[char]) -> bool {
        match (padrao.first(), nome.first()) {
            (None, None) => true,
            (Some('*'), _) => {
                glob_casa(&padrao[1..], nome)
                    || (!nome.is_empty() && glob_casa(padrao, &nome[1..]))
            }
            (Some('?'), Some(_)) => glob_casa(&padrao[1..], &nome[1..]),
            (Some(p), Some(n)) if p == n => glob_casa(&padrao[1..], &nome[1..]),
            _ => false,
        }
    }

    pub fn glob(padrao: &str) -> io::Result<Vec<String>> {
        let caminho_padrao = Path::new(padrao);
        let dir = caminho_padrao.parent().filter(|p| !p.as_os_str().is_empty());
        let nome_padrao: Vec<char> = caminho_padrao
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
            .chars()
            .collect();

        let dir_busca = dir
            .map(|d| d.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let mut resultado = Vec::new();
        for entrada in fs::read_dir(&dir_busca)? {
            let entrada = entrada?;
            let nome = entrada.file_name().to_string_lossy().into_owned();
            let nome_chars: Vec<char> = nome.chars().collect();
            if glob_casa(&nome_padrao, &nome_chars) {
                resultado.push(match dir {
                    Some(d) => d.join(&nome).to_string_lossy().into_owned(),
                    None => nome,
                });
            }
        }
        Ok(resultado)
    }

    // ---------------- Stat / permissões / links (Linux/Unix) ---------------
    #[cfg(unix)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Stat {
        pub size: u64,
        pub mtime: i64,
        pub mode: u32,
    }

    #[cfg(unix)]
    pub fn stat<P: AsRef<Path>>(caminho: P) -> io::Result<Stat> {
        let meta = fs::metadata(caminho)?;
        Ok(Stat {
            size: meta.size(),
            mtime: meta.mtime(),
            mode: meta.mode() & 0o7777,
        })
    }

    #[cfg(unix)]
    pub fn chmod<P: AsRef<Path>>(caminho: P, modo: u32) -> io::Result<()> {
        let mut perms = fs::metadata(&caminho)?.permissions();
        perms.set_mode(modo);
        fs::set_permissions(caminho, perms)
    }

    #[cfg(unix)]
    pub fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(alvo: P, link: Q) -> io::Result<()> {
        unix_symlink(alvo, link)
    }

    pub fn readlink<P: AsRef<Path>>(link: P) -> io::Result<String> {
        Ok(fs::read_link(link)?.to_string_lossy().into_owned())
    }
}

// ============================================================================
// 11) Números Aleatórios — módulo `random` do Python em std pura (v1.4.0)
// ============================================================================
pub mod random_utils {
    //! Equivalente ao módulo `random` do Python sem depender de crates externas.
    //! Usa um gerador **SplitMix64 / Xorshift64\*** rápido e thread-safe,
    //! inicializado automaticamente com entropia do relógio de alta resolução
    //! e ID do processo/thread, ou manualmente via [`seed`] para reprodutibilidade.
    //!
    //! | Python                      | Aqui                         |
    //! |-----------------------------|------------------------------|
    //! | `random.seed(s)`            | `seed(s)`                    |
    //! | `random.random()`           | `random()` → `f64` em `[0,1)`|
    //! | `random.randint(a, b)`      | `randint(a, b)` (inclusivo!) |
    //! | `random.choice(seq)`        | `choice(&seq)`               |
    //! | `random.shuffle(seq)`       | `shuffle(&mut seq)`          |

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static ESTADO_RNG: AtomicU64 = AtomicU64::new(0);

    fn semente_automatica() -> u64 {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        let pid = std::process::id() as u64;
        let end_pilha = &nanos as *const u64 as usize as u64;
        let mut z = nanos ^ pid.rotate_left(17) ^ end_pilha.rotate_right(11);
        if z == 0 {
            z = 0xCAFE_BABE_DEAD_BEEF;
        }
        z
    }

    /// Avança o estado interno usando o passo de mistura SplitMix64 e devolve `u64`.
    fn next_u64() -> u64 {
        let mut atual = ESTADO_RNG.load(Ordering::Relaxed);
        if atual == 0 {
            let inicial = semente_automatica();
            let _ = ESTADO_RNG.compare_exchange(
                0,
                inicial,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
            atual = ESTADO_RNG.load(Ordering::Relaxed);
        }
        loop {
            let proximo = atual.wrapping_add(0x9E37_79B9_7F4A_7C15);
            if ESTADO_RNG
                .compare_exchange_weak(atual, proximo, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                let mut z = proximo;
                z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
                return z ^ (z >> 31);
            }
            atual = ESTADO_RNG.load(Ordering::Relaxed);
        }
    }

    /// `random.seed(valor)` — define uma semente fixa para resultados determinísticos.
    #[inline]
    pub fn seed(valor: u64) {
        let s = if valor == 0 { 1 } else { valor };
        ESTADO_RNG.store(s, Ordering::Relaxed);
    }

    /// `random.random()` — devolve um `f64` uniforme no intervalo semiaberto `[0.0, 1.0)`.
    #[inline]
    pub fn random() -> f64 {
        // Usa os 53 bits mais significativos (precisão exata da mantissa IEEE-754 f64)
        ((next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    /// `random.randint(low, high)` — devolve um inteiro aleatório `N` tal que
    /// `low <= N <= high` (ambos inclusivos, igual ao `randint` do Python!).
    ///
    /// # Panics
    /// Causa panic se `low > high` (`ValueError` no Python).
    pub fn randint(low: i64, high: i64) -> i64 {
        assert!(
            low <= high,
            "ValueError: intervalo vazio em randint({low}, {high})"
        );
        let largura = (high as i128 - low as i128 + 1) as u128;
        let sorteado = (next_u64() as u128) % largura;
        (low as i128 + sorteado as i128) as i64
    }

    /// `random.choice(seq)` — escolhe um elemento aleatório de uma fatia (`None` se vazia).
    #[inline]
    pub fn choice<T>(fatia: &[T]) -> Option<&T> {
        if fatia.is_empty() {
            None
        } else {
            let idx = (next_u64() as usize) % fatia.len();
            Some(&fatia[idx])
        }
    }

    /// `random.shuffle(seq)` — embaralha uma fatia ou `Vec<T>` in-place usando
    /// o algoritmo de Fisher-Yates em $O(N)$.
    pub fn shuffle<T>(fatia: &mut [T]) {
        let n = fatia.len();
        for i in (1..n).rev() {
            let j = (next_u64() as usize) % (i + 1);
            fatia.swap(i, j);
        }
    }
}

// ============================================================================
// 12) Cyber Security & Binary Analysis — pub mod sec_utils (v1.5.0)
// ============================================================================
pub mod sec_utils {
    //! Utilitários de Segurança Ofensiva/Defensiva (Red & Purple Team), Engenharia
    //! Reversa e Exploit Development equivalentes ao `pwntools`, `struct` e `binascii`:
    //!
    //! - **Packing/Unpacking:** `p8`, `p16`, `p32`, `p64`, `u8_le`..`u64_le`, `p16_be`..`u64_be`.
    //! - **Inspeção de Memória:** `hexdump` (formato canônico `xxd -C`).
    //! - **Codificação:** `hex_encode`, `hex_decode`, `base64_encode`, `base64_decode`.
    //! - **Ofuscação / Criptoanálise:** `xor_cyclic`, `xor_single`, `shannon_entropy`, `is_high_entropy`.
    //! - **Buffer Overflow Offsets:** `cyclic` e `cyclic_find` (sequência De Bruijn).
    //! - **Segurança de Memória:** `constant_time_eq` e `secure_zero`.

    use std::fmt::Write as _;

    // ---------------- Packing / Unpacking (Little-Endian & Big-Endian) -----

    /// Empacota `u8` em `[u8; 1]` (`pwntools.p8`).
    #[inline]
    pub const fn p8(v: u8) -> [u8; 1] {
        [v]
    }

    /// Empacota `u16` em Little-Endian `[u8; 2]` (`pwntools.p16`).
    #[inline]
    pub const fn p16(v: u16) -> [u8; 2] {
        v.to_le_bytes()
    }

    /// Empacota `u32` em Little-Endian `[u8; 4]` (`pwntools.p32`).
    #[inline]
    pub const fn p32(v: u32) -> [u8; 4] {
        v.to_le_bytes()
    }

    /// Empacota `u64` em Little-Endian `[u8; 8]` (`pwntools.p64`).
    #[inline]
    pub const fn p64(v: u64) -> [u8; 8] {
        v.to_le_bytes()
    }

    /// Empacota `u16` em Big-Endian (Network Byte Order) `[u8; 2]`.
    #[inline]
    pub const fn p16_be(v: u16) -> [u8; 2] {
        v.to_be_bytes()
    }

    /// Empacota `u32` em Big-Endian (Network Byte Order) `[u8; 4]`.
    #[inline]
    pub const fn p32_be(v: u32) -> [u8; 4] {
        v.to_be_bytes()
    }

    /// Empacota `u64` em Big-Endian (Network Byte Order) `[u8; 8]`.
    #[inline]
    pub const fn p64_be(v: u64) -> [u8; 8] {
        v.to_be_bytes()
    }

    /// Desempacota `u8` a partir do primeiro byte da fatia.
    #[inline]
    pub fn u8_le(b: &[u8]) -> Option<u8> {
        b.first().copied()
    }

    /// Desempacota `u16` em Little-Endian (`pwntools.u16`).
    #[inline]
    pub fn u16_le(b: &[u8]) -> Option<u16> {
        if b.len() < 2 {
            None
        } else {
            Some(u16::from_le_bytes([b[0], b[1]]))
        }
    }

    /// Desempacota `u32` em Little-Endian (`pwntools.u32`).
    #[inline]
    pub fn u32_le(b: &[u8]) -> Option<u32> {
        if b.len() < 4 {
            None
        } else {
            Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        }
    }

    /// Desempacota `u64` em Little-Endian (`pwntools.u64`).
    #[inline]
    pub fn u64_le(b: &[u8]) -> Option<u64> {
        if b.len() < 8 {
            None
        } else {
            Some(u64::from_le_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ]))
        }
    }

    /// Desempacota `u16` em Big-Endian.
    #[inline]
    pub fn u16_be(b: &[u8]) -> Option<u16> {
        if b.len() < 2 {
            None
        } else {
            Some(u16::from_be_bytes([b[0], b[1]]))
        }
    }

    /// Desempacota `u32` em Big-Endian.
    #[inline]
    pub fn u32_be(b: &[u8]) -> Option<u32> {
        if b.len() < 4 {
            None
        } else {
            Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        }
    }

    /// Desempacota `u64` em Big-Endian.
    #[inline]
    pub fn u64_be(b: &[u8]) -> Option<u64> {
        if b.len() < 8 {
            None
        } else {
            Some(u64::from_be_bytes([
                b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
            ]))
        }
    }

    // ---------------- Codificação Hex / Base64 e Hexdump --------------------

    /// Codifica uma fatia de bytes para string hexadecimal minúscula (`binascii.hexlify`).
    pub fn hex_encode(dados: &[u8]) -> String {
        const TABELA: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(dados.len() * 2);
        for &b in dados {
            out.push(TABELA[(b >> 4) as usize] as char);
            out.push(TABELA[(b & 0x0F) as usize] as char);
        }
        out
    }

    fn val_nibble(c: u8) -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    }

    /// Decodifica uma string hexadecimal em `Vec<u8>` (`binascii.unhexlify` / `pwntools.unhex`).
    /// Ignora espaços em branco, quebras de linha e prefixos `0x` ou `\x`.
    pub fn hex_decode(hex_str: &str) -> Result<Vec<u8>, String> {
        let mut limpo = Vec::with_capacity(hex_str.len());
        let bytes = hex_str.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            if b.is_ascii_whitespace() || b == b':' || b == b'-' {
                i += 1;
                continue;
            }
            if (b == b'0' || b == b'\\')
                && i + 1 < bytes.len()
                && (bytes[i + 1] == b'x' || bytes[i + 1] == b'X')
            {
                i += 2;
                continue;
            }
            limpo.push(b);
            i += 1;
        }

        if limpo.len() % 2 != 0 {
            return Err(format!(
                "ValueError: string hexadecimal com número ímpar de dígitos ({})",
                limpo.len()
            ));
        }

        let mut saida = Vec::with_capacity(limpo.len() / 2);
        for par in limpo.chunks_exact(2) {
            let hi = val_nibble(par[0]).ok_or_else(|| {
                format!(
                    "ValueError: caractere inválido '{}' em hex_decode",
                    par[0] as char
                )
            })?;
            let lo = val_nibble(par[1]).ok_or_else(|| {
                format!(
                    "ValueError: caractere inválido '{}' em hex_decode",
                    par[1] as char
                )
            })?;
            saida.push((hi << 4) | lo);
        }
        Ok(saida)
    }

    /// Gera um dump hexadecimal canônico estilo `xxd -C` / `hexdump -C` / `pwntools.hexdump`,
    /// exibindo offset de 8 dígitos, duas colunas de 8 bytes e representação ASCII lateral.
    pub fn hexdump(dados: &[u8]) -> String {
        if dados.is_empty() {
            return String::new();
        }
        let mut out = String::with_capacity((dados.len() / 16 + 1) * 78);
        for (idx_bloco, fatia) in dados.chunks(16).enumerate() {
            let offset = idx_bloco * 16;
            let _ = write!(out, "{offset:08x}  ");

            for i in 0..16 {
                if i == 8 {
                    out.push(' ');
                }
                if i < fatia.len() {
                    let _ = write!(out, "{:02x} ", fatia[i]);
                } else {
                    out.push_str("   ");
                }
            }

            out.push_str(" |");
            for &b in fatia {
                if (0x20..=0x7E).contains(&b) {
                    out.push(b as char);
                } else {
                    out.push('.');
                }
            }
            out.push('|');
            if offset + 16 < dados.len() {
                out.push('\n');
            }
        }
        out
    }

    /// Codifica uma fatia de bytes em Base64 padrão (RFC 4648).
    pub fn base64_encode(dados: &[u8]) -> String {
        const ALFABETO: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity(((dados.len() + 2) / 3) * 4);
        for bloco in dados.chunks(3) {
            let b0 = bloco[0] as u32;
            let b1 = if bloco.len() > 1 { bloco[1] as u32 } else { 0 };
            let b2 = if bloco.len() > 2 { bloco[2] as u32 } else { 0 };
            let triplo = (b0 << 16) | (b1 << 8) | b2;

            out.push(ALFABETO[((triplo >> 18) & 0x3F) as usize] as char);
            out.push(ALFABETO[((triplo >> 12) & 0x3F) as usize] as char);
            if bloco.len() > 1 {
                out.push(ALFABETO[((triplo >> 6) & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
            if bloco.len() > 2 {
                out.push(ALFABETO[(triplo & 0x3F) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

    /// Decodifica uma string Base64 padrão (RFC 4648) em `Vec<u8>`.
    pub fn base64_decode(b64: &str) -> Result<Vec<u8>, String> {
        let limpo: Vec<u8> = b64
            .bytes()
            .filter(|b| !b.is_ascii_whitespace())
            .collect();
        if limpo.is_empty() {
            return Ok(Vec::new());
        }
        if limpo.len() % 4 != 0 {
            return Err("ValueError: comprimento de Base64 inválido".to_string());
        }
        fn val_b64(c: u8) -> Option<u8> {
            match c {
                b'A'..=b'Z' => Some(c - b'A'),
                b'a'..=b'z' => Some(c - b'a' + 26),
                b'0'..=b'9' => Some(c - b'0' + 52),
                b'+' => Some(62),
                b'/' => Some(63),
                _ => None,
            }
        }
        let mut out = Vec::with_capacity((limpo.len() / 4) * 3);
        for quad in limpo.chunks_exact(4) {
            let v0 = val_b64(quad[0]).ok_or("ValueError: caractere Base64 inválido")? as u32;
            let v1 = val_b64(quad[1]).ok_or("ValueError: caractere Base64 inválido")? as u32;
            let v2 = if quad[2] == b'=' {
                0
            } else {
                val_b64(quad[2]).ok_or("ValueError: caractere Base64 inválido")? as u32
            };
            let v3 = if quad[3] == b'=' {
                0
            } else {
                val_b64(quad[3]).ok_or("ValueError: caractere Base64 inválido")? as u32
            };

            let triplo = (v0 << 18) | (v1 << 12) | (v2 << 6) | v3;
            out.push(((triplo >> 16) & 0xFF) as u8);
            if quad[2] != b'=' {
                out.push(((triplo >> 8) & 0xFF) as u8);
            }
            if quad[3] != b'=' {
                out.push((triplo & 0xFF) as u8);
            }
        }
        Ok(out)
    }

    // ---------------- XOR Cíclico & Entropia de Shannon --------------------

    /// Aplica XOR cíclico entre `dados` e `chave` (`pwntools.xor`).
    /// Se `chave` for vazia, devolve uma cópia inalterada de `dados`.
    pub fn xor_cyclic(dados: &[u8], chave: &[u8]) -> Vec<u8> {
        if chave.is_empty() {
            return dados.to_vec();
        }
        dados
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ chave[i % chave.len()])
            .collect()
    }

    /// Aplica XOR com um único byte sobre toda a fatia.
    #[inline]
    pub fn xor_single(dados: &[u8], chave: u8) -> Vec<u8> {
        dados.iter().map(|&b| b ^ chave).collect()
    }

    /// Calcula a Entropia de Shannon ($H(X) = -\sum P(x_i) \log_2 P(x_i)$) em bits por byte
    /// no intervalo `[0.0, 8.0]`, usando contagem estática na stack ($O(N)$ e zero alocação).
    /// Essencial para detectar binários empacotados (packers), shellcodes cifrados ou tráfego C2.
    pub fn shannon_entropy(dados: &[u8]) -> f64 {
        if dados.is_empty() {
            return 0.0;
        }
        let mut freq = [0usize; 256];
        for &b in dados {
            freq[b as usize] += 1;
        }
        let total = dados.len() as f64;
        let mut entropia = 0.0;
        for &contagem in &freq {
            if contagem > 0 {
                let p = (contagem as f64) / total;
                entropia -= p * p.log2();
            }
        }
        entropia
    }

    /// Verifica se a entropia de Shannon dos dados é maior ou igual a `limiar`
    /// (ex.: `limiar = 7.2` para detectar criptografia/compressão forte).
    #[inline]
    pub fn is_high_entropy(dados: &[u8], limiar: f64) -> bool {
        shannon_entropy(dados) >= limiar
    }

    // ---------------- Padrão Cíclico De Bruijn (Buffer Overflow Offset) -----

    /// Gera um padrão cíclico não-repetitivo de 4 bytes (estilo `pwntools.cyclic`)
    /// para determinar o offset exato de sobrescrita de registradores (EIP/RIP).
    pub fn cyclic(tamanho: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(tamanho);
        'externo: for c1 in b'a'..=b'z' {
            for c2 in b'a'..=b'z' {
                for c3 in b'a'..=b'z' {
                    for c4 in b'a'..=b'z' {
                        if c1 == c2 && c2 == c3 && c3 == c4 {
                            continue;
                        }
                        for &b in &[c1, c2, c3, c4] {
                            out.push(b);
                            if out.len() >= tamanho {
                                break 'externo;
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// Localiza o offset exato de uma subsequência de 4 bytes (ou `u32` little-endian)
    /// dentro do padrão gerado por [`cyclic`] (`pwntools.cyclic_find`).
    pub fn cyclic_find(subpadrao: &[u8]) -> Option<usize> {
        if subpadrao.is_empty() {
            return None;
        }
        let padrao = cyclic(20_000);
        padrao
            .windows(subpadrao.len())
            .position(|janela| janela == subpadrao)
    }

    // ---------------- Segurança de Memória & Criptografia ------------------

    /// Compara duas fatias de bytes em tempo constante (imune a *timing attacks*
    /// na verificação de HMACs, tokens e senhas).
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut diff: u8 = 0;
        for (&x, &y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }

    /// Zera um buffer sensível em memória usando escrita volátil (`write_volatile`)
    /// + barreira de compilador, impedindo que o otimizador LLVM remova a limpeza.
    pub fn secure_zero(buf: &mut [u8]) {
        for b in buf.iter_mut() {
            unsafe {
                std::ptr::write_volatile(b, 0u8);
            }
        }
        std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
    }
}

// ============================================================================
// Macros de Produtividade CyberSec (Red / Purple Team) — v1.5.0
// ============================================================================

/// Converte uma string hexadecimal (aceitando espaços, `0x` ou `\x`) em `Vec<u8>`.
/// Equivalente ao `unhex("dead beef")` do `pwntools`.
#[macro_export]
macro_rules! unhex {
    ($hex:expr) => {
        $crate::sec_utils::hex_decode($hex)
            .unwrap_or_else(|e| panic!("{e}"))
    };
}

/// Realiza um probe rápido em `(host, port)` com timeout padrão de `1.0s` (ou customizado)
/// e `grab_banner = true`, devolvendo [`net_utils::PortScanResult`].
#[macro_export]
macro_rules! tcp_probe {
    ($host:expr, $port:expr) => {
        $crate::net_utils::tcp_probe($host, $port, 1.0, true)
    };
    ($host:expr, $port:expr, $timeout:expr) => {
        $crate::net_utils::tcp_probe($host, $port, $timeout, true)
    };
}

/// Sobe um servidor TCP de teste/honeypot em background numa thread dedicada para
/// atender uma conexão, devolvendo `(SocketAddr, JoinHandle<()>)`.
/// Ideal para simulações de Purple Team, validação de exploits e testes de loopback.
#[macro_export]
macro_rules! tcp_server_spawn {
    ($addr:expr, |$cli:ident, $peer:ident| $body:block) => {{
        let srv = $crate::net_utils::TcpServer::listen($addr)
            .expect("IOError: falha ao iniciar TcpServer em tcp_server_spawn!");
        let local = srv
            .local_addr()
            .expect("IOError: falha ao obter local_addr em tcp_server_spawn!");
        let handle = ::std::thread::spawn(move || {
            if let Ok((mut $cli, $peer)) = srv.accept() {
                $body
            }
        });
        (local, handle)
    }};
    ($addr:expr, |$cli:ident| $body:block) => {{
        $crate::tcp_server_spawn!($addr, |$cli, _peer| $body)
    }};
}

// ============================================================================
// Prelude — import único e ergonômico: `use pyrsutils::prelude::*;`
// ============================================================================
pub mod prelude {
    pub use super::arithmetic::{abs, div, divmod, mult, pow, root, sub, sum, ArithError, NumeroPy};
    pub use super::builtins::{
        all, any, enumerate, histogram, len, max, min, print, reversed, sorted, type_of, zip,
        PyLen,
    };
    pub use super::list_utils::PyList;
    pub use super::matrix_utils::Matrix;
    pub use super::range_utils::{range, range_from, range_step, PyRange};
    pub use super::string_utils::{
        count, endswith, find, is_palindrome, join, lower, replace, split, startswith, strip,
        upper,
    };
    pub use super::{input, try_input, InputError};
    // v1.1 — tempo e rede
    pub use super::net_utils::{getaddrinfo, gethostbyname, TcpClient, TcpServer, UdpSock};
    pub use super::time_utils::{
        gmtime, gmtime_now, mktime_utc, perf_counter, perf_counter_ns, sleep, strftime, time,
        timeit, timeit_repeat, StructTime,
    };
    // v1.2 / v1.4 — sistema de arquivos + I/O rápido de texto
    pub use super::fs_utils::{
        abspath, append_text, basename, chdir, copy, dirname, exists, expanduser, getcwd, glob,
        isdir, isfile, islink, join_path, listdir, makedirs, mkdir, read_text, readlines,
        readlink, realpath, remove, rename, rmdir, rmtree, splitext, walk, write_text, Walk,
        WalkItem,
    };
    #[cfg(unix)]
    pub use super::fs_utils::{chmod, stat, symlink, Stat};
    // v1.4 — números aleatórios
    pub use super::random_utils::{choice, randint, random, seed, shuffle};
    // v1.5 — Cyber Security, Exploit Dev, Packet Crafting, PCAP & Linux /proc
    pub use super::net_utils::{
        ipv4_tcp_checksum, parse_cidr, parse_ports, parse_proc_net_arp_str,
        parse_proc_net_tcp_str, proc_net_arp, proc_net_tcp, rfc1071_checksum, tcp_probe, tcp_scan,
        Ipv4Cidr, Ipv4TcpPacket, PcapWriter, PortScanResult, ProcArpEntry, ProcTcpEntry, TCP_ACK,
        TCP_CWR, TCP_ECE, TCP_FIN, TCP_PSH, TCP_RST, TCP_SYN, TCP_URG,
    };
    pub use super::sec_utils::{
        base64_decode, base64_encode, constant_time_eq, cyclic, cyclic_find, hex_decode,
        hex_encode, hexdump, is_high_entropy, p16, p16_be, p32, p32_be, p64, p64_be, p8,
        secure_zero, shannon_entropy, u16_be, u16_le, u32_be, u32_le, u64_be, u64_le, u8_le,
        xor_cyclic, xor_single,
    };
}

// ============================================================================
// Suíte Completa de Testes (v1.0 + v1.1 + v1.2 + v1.4.0 + Novos Testes v1.5.0)
// ============================================================================
#[cfg(test)]
mod tests {
    #[test]
    fn string_sempre_converte() {
        let texto = "ola mundo".to_string();
        let r: Result<String, _> = texto.parse();
        assert_eq!(r.unwrap(), "ola mundo");
    }

    #[test]
    fn inteiro_converte() {
        let r: Result<i32, _> = "42".parse();
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn float_converte() {
        let r: Result<f64, _> = "3.14".parse();
        assert_eq!(r.unwrap(), 3.14);
    }

    #[test]
    fn bool_converte() {
        let r: Result<bool, _> = "true".parse();
        assert_eq!(r.unwrap(), true);
    }

    #[test]
    fn inteiro_invalido_falha() {
        let r: Result<i32, _> = "abc".parse();
        assert!(r.is_err());
    }
}

#[cfg(test)]
mod tests_extensoes {
    use super::*;

    #[test]
    fn range_basico_e_contains_funcionam() {
        let r = range_utils::range(5);
        assert!(r.contains(3));
        assert!(!r.contains(5));
        let v: Vec<i64> = r.collect();
        assert_eq!(v, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn range_step_negativo_funciona() {
        let r = range_utils::range_step(10, 0, -2);
        assert!(r.contains(8));
        assert!(!r.contains(7));
        let v: Vec<i64> = r.collect();
        assert_eq!(v, vec![10, 8, 6, 4, 2]);
    }

    #[test]
    fn range_tem_len_sem_iterar() {
        use builtins::len;
        assert_eq!(len(&range_utils::range(10)), 10);
        assert_eq!(len(&range_utils::range_step(10, 0, -2)), 5);
        assert_eq!(len(&range_utils::range_from(5, 5)), 0);
        assert_eq!(len(&range_utils::range_from(5, 2)), 0);
    }

    #[test]
    fn lista_append_pop_remove_extend_index() {
        use list_utils::PyList;
        let mut v = vec![1, 2, 3];
        v.append_item(4);
        assert_eq!(v, vec![1, 2, 3, 4]);
        assert_eq!(v.pop_at(None), Some(4));
        assert_eq!(v.pop_at(Some(0)), Some(1));
        assert_eq!(v, vec![2, 3]);
        assert!(v.remove_value(&3));
        assert_eq!(v, vec![2]);
        v.extend_from(vec![5, 6, 2]);
        assert_eq!(v.index_of(&5), Some(1));
        assert_eq!(v.count_value(&2), 2);
    }

    #[test]
    fn matriz_get_set_transpose_e_dot() {
        use matrix_utils::Matrix;
        let mut m = Matrix::new(2, 3, 0);
        m[(0, 0)] = 1;
        m[(1, 2)] = 9;
        assert_eq!(m.get(0, 0), Some(&1));
        assert_eq!(m.get(1, 2), Some(&9));
        let t = m.transpose();
        assert_eq!(t[(0, 0)], 1);
        assert_eq!(t[(2, 1)], 9);

        // Multiplicação matricial (dot): (2x3) * (3x2) = (2x2)
        let prod = m.dot(&t).unwrap();
        assert_eq!((prod.linhas, prod.colunas), (2, 2));
        assert_eq!(prod[(0, 0)], 1);
        assert_eq!(prod[(1, 1)], 81);
    }

    #[test]
    fn builtins_completos() {
        use builtins::*;
        assert_eq!(min(vec![3, 1, 2]), Some(1));
        assert_eq!(max(vec![3, 1, 2]), Some(3));
        assert_eq!(sorted(vec![3, 1, 2]), vec![1, 2, 3]);
        let r: Vec<i32> = reversed(vec![1, 2, 3]).collect();
        assert_eq!(r, vec![3, 2, 1]);
        assert_eq!(len("olá"), 3);
        assert_eq!(len(&String::from("café")), 4);
        assert_eq!(len(&vec![10, 20]), 2);

        let pares: Vec<(usize, &str)> = enumerate(vec!["a", "b"]).collect();
        assert_eq!(pares, vec![(0, "a"), (1, "b")]);

        let zipado: Vec<(i32, i32)> = zip(vec![1, 2], vec![10, 20]).collect();
        assert_eq!(zipado, vec![(1, 10), (2, 20)]);

        assert!(any(vec![false, true, false]));
        assert!(all(vec![true, true]));

        let hist = histogram("banana".chars());
        assert_eq!(hist.get(&'a'), Some(&3));
        assert_eq!(hist.get(&'n'), Some(&2));
    }

    #[test]
    fn aritmetica_basica_e_erros_de_borda() {
        use arithmetic::*;
        assert_eq!(sum(vec![1usize, 2, 3]), 6usize);
        assert_eq!(sub(vec![10isize, 1, 2]), Some(7isize));
        assert_eq!(mult(vec![2, 3, 4]), 24);
        assert_eq!(div(10, 2), Ok(5));
        assert_eq!(div(10, 0), Err(ArithError::DivisaoPorZero));
        assert_eq!(divmod(7, 3), Ok((2, 1)));
        assert_eq!(abs(-42i32), 42);
        assert_eq!(pow(2, 10), 1024);
        assert!((root(9.0, 2.0).unwrap() - 3.0).abs() < 1e-9);
        assert!((root(-8.0, 3.0).unwrap() + 2.0).abs() < 1e-9);
        assert_eq!(root(-8.0, 2.5), Err(ArithError::RaizDeNegativoComIndicePar));
        assert_eq!(root(9.0, 0.0), Err(ArithError::IndiceDeRaizInvalido));
    }

    #[test]
    fn strings_utilitarios_completos() {
        use string_utils::*;
        assert_eq!(split("a,b,c", ","), vec!["a", "b", "c"]);
        assert_eq!(strip("  oi  "), "oi");
        assert_eq!(join(", ", vec!["a", "b", "c"]), "a, b, c");
        assert_eq!(join(", ", Vec::<&str>::new()), "");
        assert_eq!(replace("banana", "na", "NA"), "baNANA");
        assert_eq!(lower("OlÁ"), "olá");
        assert_eq!(upper("olá"), "OLÁ");
        assert!(startswith("rust_lang", "rust"));
        assert!(endswith("rust_lang", "lang"));
        assert_eq!(find("café_bom", "bom"), 5);
        assert_eq!(find("abc", "z"), -1);
        assert_eq!(count("banana", "na"), 2);
        assert!(is_palindrome("reviver"));
        assert!(is_palindrome("osso"));
        assert!(!is_palindrome("python"));
    }

    class! {
        struct Contador {
            valor: i32
        }
        impl {
            fn incrementar(&mut self) {
                self.valor += 1;
            }
            fn valor_atual(&self) -> i32 {
                self.valor
            }
        }
    }

    class! {
        struct ContadorComPasso : Contador {
            passo: i32
        }
        impl {
            fn avancar(&mut self) {
                self.valor += self.passo;
            }
        }
    }

    #[test]
    fn class_macro_base_e_heranca_funcionam() {
        let mut c = Contador::new(0);
        c.incrementar();
        c.incrementar();
        assert_eq!(c.valor_atual(), 2);

        // Testa herança estilo C++ (ContadorComPasso : Contador)
        let mut cp = ContadorComPasso::new(Contador::new(10), 5);
        cp.incrementar(); // Método herdado da base via DerefMut!
        cp.avancar();     // Método próprio da filha
        assert_eq!(cp.valor_atual(), 16);
    }
}

#[cfg(test)]
mod tests_tempo_rede_random {
    use super::net_utils::*;
    use super::random_utils::*;
    use super::time_utils::*;

    #[test]
    fn time_e_perf_counter_avancam() {
        assert!(time() > 1.7e9);
        let t0 = perf_counter();
        sleep(0.02);
        assert!(perf_counter() - t0 >= 0.019);
        let n0 = perf_counter_ns();
        assert!(perf_counter_ns() >= n0);
    }

    #[test]
    fn macro_timeit_higienica_funciona() {
        let mut c = 0;
        let t = crate::timeit!(7, {
            c += 1;
        });
        assert_eq!(c, 7);
        assert!(t >= 0.0);
    }

    #[test]
    fn gmtime_e_strftime() {
        let t = gmtime(1_000_000_000);
        assert_eq!(strftime("%Y-%m-%d %H:%M:%S", &t), "2001-09-09 01:46:40");
        assert_eq!(mktime_utc(&t), 1_000_000_000);
    }

    #[test]
    fn tcp_e_udp_loopback() {
        let srv = TcpServer::listen("127.0.0.1:0").unwrap();
        let porta = srv.local_addr().unwrap().port();
        let th = std::thread::spawn(move || {
            let (mut cli, _) = srv.accept().unwrap();
            let d = cli.recv_exact(5).unwrap();
            cli.send(&d).unwrap();
        });
        let mut c = TcpClient::connect(("127.0.0.1", porta)).unwrap();
        c.set_timeout(Some(2.0)).unwrap();
        c.send("olá!").unwrap();
        assert_eq!(c.recv_exact(5).unwrap(), "olá!".as_bytes());
        th.join().unwrap();

        let a = UdpSock::bind("127.0.0.1:0").unwrap();
        let b = UdpSock::bind("127.0.0.1:0").unwrap();
        b.send_to("ping", a.local_addr().unwrap()).unwrap();
        let (msg, _) = a.recv_from(512).unwrap();
        assert_eq!(msg, b"ping");
    }

    #[test]
    fn random_seed_randint_choice_shuffle() {
        seed(42);
        for _ in 0..50 {
            let r = random();
            assert!((0.0..1.0).contains(&r));
            let n = randint(5, 10);
            assert!((5..=10).contains(&n));
        }
        let itens = [10, 20, 30, 40];
        assert!(choice(&itens).is_some());
        let mut v = vec![1, 2, 3, 4, 5, 6, 7, 8];
        shuffle(&mut v);
        assert_eq!(v.len(), 8);
    }
}

#[cfg(test)]
mod tests_fs {
    use super::fs_utils::*;
    use std::fs;

    fn area_de_teste(sufixo: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pyrsutils_v14_fs_{}_{}",
            std::process::id(),
            sufixo
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn io_texto_e_walk_seguro_contra_loop_symlink() {
        let dir = area_de_teste("io_walk");
        let arq = dir.join("notas.txt");
        write_text(&arq, "linha 1\n").unwrap();
        append_text(&arq, "linha 2\n").unwrap();
        assert_eq!(read_text(&arq).unwrap(), "linha 1\nlinha 2\n");
        assert_eq!(readlines(&arq).unwrap(), vec!["linha 1", "linha 2"]);

        #[cfg(unix)]
        {
            // Cria symlink apontando para o próprio diretório pai (ciclo!)
            let link_ciclico = dir.join("loop_infinito");
            symlink(&dir, &link_ciclico).unwrap();
        }

        // O walk não deve entrar em loop infinito porque não segue symlinks
        let mut passos = 0;
        for item in walk(&dir) {
            let _ = item.unwrap();
            passos += 1;
            assert!(passos < 10, "walk entrou em loop infinito com symlink!");
        }
        rmtree(&dir).unwrap();
    }
}

#[cfg(test)]
mod tests_v15_cybersec_e_rede {
    use super::net_utils::*;
    use super::sec_utils::*;
    use std::net::Ipv4Addr;

    #[test]
    fn pwntools_recvuntil_sendlineafter_sem_overread() {
        let (addr, handle) = crate::tcp_server_spawn!("127.0.0.1:0", |cli| {
            // Envia banner + prompt + dado extra colado no mesmo buffer TCP
            cli.send(b"Bem-vindo ao CTF\nSenha: ").unwrap();
            let tentativa = cli.recvline_string(128).unwrap();
            if tentativa == "s3cr3t" {
                cli.send(b"FLAG{pwn_ok}\nRESTO_DO_STREAM").unwrap();
            }
        });

        let mut p = TcpClient::connect(addr).unwrap();
        p.set_timeout(Some(2.0)).unwrap();

        let prompt = p.sendlineafter(b"Senha: ", b"s3cr3t", 256).unwrap();
        assert!(prompt.ends_with(b"Senha: "));

        let flag = p.recvline_string(128).unwrap();
        assert_eq!(flag, "FLAG{pwn_ok}");

        // Garante que recvline não engoliu bytes além do '\n'!
        let resto = p.recv_exact(15).unwrap();
        assert_eq!(&resto, b"RESTO_DO_STREAM");
        handle.join().unwrap();
    }

    #[test]
    fn cidr_ipv4_e_parse_ports_nmap() {
        let cidr = parse_cidr("192.168.10.0/30").unwrap();
        assert_eq!(cidr.network(), Ipv4Addr::new(192, 168, 10, 0));
        assert_eq!(cidr.broadcast(), Ipv4Addr::new(192, 168, 10, 3));
        assert_eq!(cidr.netmask(), Ipv4Addr::new(255, 255, 255, 252));
        assert!(cidr.contains(Ipv4Addr::new(192, 168, 10, 2)));
        assert!(!cidr.contains(Ipv4Addr::new(192, 168, 10, 4)));

        let hosts: Vec<Ipv4Addr> = cidr.hosts().collect();
        assert_eq!(
            hosts,
            vec![
                Ipv4Addr::new(192, 168, 10, 1),
                Ipv4Addr::new(192, 168, 10, 2)
            ]
        );

        let portas = parse_ports("443, 22, 8000-8002, 22").unwrap();
        assert_eq!(portas, vec![22, 443, 8000, 8001, 8002]);
    }

    #[test]
    fn tcp_scan_concorrente_e_banner_grabbing() {
        let (addr, handle) = crate::tcp_server_spawn!("127.0.0.1:0", |cli| {
            let _ = cli.send(b"SSH-2.0-OpenSSH_9.6\r\n");
        });

        let porta_aberta = addr.port();
        let porta_fechada = if porta_aberta > 2000 {
            porta_aberta - 1
        } else {
            porta_aberta + 1
        };

        let resultados = tcp_scan(
            "127.0.0.1",
            &[porta_fechada, porta_aberta],
            0.5,
            2,
            true,
        );
        assert_eq!(resultados.len(), 2);

        let r_open = resultados.iter().find(|r| r.port == porta_aberta).unwrap();
        assert!(r_open.open);
        assert_eq!(r_open.banner.as_deref(), Some("SSH-2.0-OpenSSH_9.6"));
        handle.join().unwrap();
    }

    #[test]
    fn packet_crafting_ipv4_tcp_checksum_e_pcap_writer() {
        let src = Ipv4Addr::new(10, 0, 0, 1);
        let dst = Ipv4Addr::new(10, 0, 0, 2);
        let pkt = Ipv4TcpPacket::new(src, dst, 4444, 80, TCP_SYN | TCP_ACK)
            .with_seq(0x11223344)
            .with_ack(0x55667788)
            .with_payload(b"GET / HTTP/1.0\r\n\r\n");

        let raw = pkt.build();
        // O checksum RFC 1071 sobre um cabeçalho IPv4 íntegro deve resultar em 0
        assert_eq!(rfc1071_checksum(&raw[0..20]), 0);
        assert_eq!(ipv4_tcp_checksum(src, dst, &raw[20..]), 0);

        let dissecado = Ipv4TcpPacket::parse(&raw).unwrap();
        assert_eq!(dissecado, pkt);

        let mut pcap = PcapWriter::new_raw_ip();
        pcap.add_packet(1_700_000_000, 500, &raw);
        let bytes_pcap = pcap.into_bytes();
        // 24 bytes Global Header + 16 bytes Packet Header + tamanho do pacote
        assert_eq!(bytes_pcap.len(), 24 + 16 + raw.len());
        assert_eq!(&bytes_pcap[0..4], &0xA1B2_C3D4u32.to_le_bytes());
    }

    #[test]
    fn proc_net_tcp_e_arp_parsers_funcionam() {
        let amostra_tcp = "\
  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode
   0: 0100007F:0035 00000000:0000 0A 00000000:00000000 00:00000000 00000000   101        0 20485 1 0000000000000000 100 0 0 10 0
   1: 6401A8C0:D431 0101A8C0:01BB 01 00000000:00000000 02:00000120 00000000  1000        0 44123 2 0000000000000000 20 4 30 10 -1\n";

        let entradas = parse_proc_net_tcp_str(amostra_tcp);
        assert_eq!(entradas.len(), 2);
        assert_eq!(entradas[0].local_addr.ip(), &Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(entradas[0].local_addr.port(), 53);
        assert_eq!(entradas[0].state, "LISTEN");
        assert_eq!(entradas[0].uid, 101);
        assert_eq!(entradas[0].inode, 20485);

        assert_eq!(entradas[1].local_addr.ip(), &Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(entradas[1].rem_addr.ip(), &Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(entradas[1].rem_addr.port(), 443);
        assert_eq!(entradas[1].state, "ESTABLISHED");

        let amostra_arp = "\
IP address       HW type     Flags       HW address            Mask     Device
192.168.1.1      0x1         0x2         aa:bb:cc:dd:ee:ff     *        wlan0\n";
        let arp = parse_proc_net_arp_str(amostra_arp);
        assert_eq!(arp.len(), 1);
        assert_eq!(arp[0].ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(arp[0].mac, "aa:bb:cc:dd:ee:ff");
        assert_eq!(arp[0].device, "wlan0");
    }

    #[test]
    fn sec_utils_pack_hex_base64_xor_entropia_e_cyclic() {
        assert_eq!(u64_le(&p64(0xDEAD_BEEF_CAFE_BABEu64)), Some(0xDEAD_BEEF_CAFE_BABEu64));
        assert_eq!(u32_be(&p32_be(0x1122_3344)), Some(0x1122_3344));

        let bytes = crate::unhex!("dead beef \\x90 0x41");
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF, 0x90, 0x41]);
        assert_eq!(hex_encode(&bytes), "deadbeef9041");

        let b64 = base64_encode(b"RedTeam:PurpleTeam");
        assert_eq!(b64, "UmVkVGVhbTpQdXJwbGVUZWFt");
        assert_eq!(base64_decode(&b64).unwrap(), b"RedTeam:PurpleTeam");

        let claro = b"PAYLOAD_SECRETO_C2";
        let chave = b"K3Y";
        let cifrado = xor_cyclic(claro, chave);
        assert_ne!(&cifrado, claro);
        assert_eq!(xor_cyclic(&cifrado, chave), claro);

        // Entropia de bytes repetidos = 0.0; entropia de todos os 256 valores uniformes = 8.0
        assert!((shannon_entropy(&[0x90u8; 128]) - 0.0).abs() < 1e-9);
        let todos_256: Vec<u8> = (0u8..=255u8).collect();
        assert!((shannon_entropy(&todos_256) - 8.0).abs() < 1e-9);
        assert!(is_high_entropy(&todos_256, 7.9));

        // Padrão De Bruijn (cyclic / cyclic_find)
        let padrao = cyclic(512);
        let fatia_eip = &padrao[136..140];
        assert_eq!(cyclic_find(fatia_eip), Some(136));

        // Hexdump, constant_time_eq e secure_zero
        let dump = hexdump(b"Hello Rust!\n");
        assert!(dump.starts_with("00000000  48 65 6c 6c 6f 20 52 75  73 74 21 0a"));
        assert!(dump.ends_with("|Hello Rust!.|"));

        assert!(constant_time_eq(b"hmac_valido", b"hmac_valido"));
        assert!(!constant_time_eq(b"hmac_valido", b"hmac_falso!"));

        let mut segredo = vec![0xAA, 0xBB, 0xCC, 0xDD];
        secure_zero(&mut segredo);
        assert_eq!(segredo, vec![0, 0, 0, 0]);
    }
}
