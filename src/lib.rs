// SPDX-License-Identifier: Apache-2.0

//! This crate implements a macro that defines ELF notes for communicating
//! information to the tooling that processes ELF notes.
//!
//! Be sure to check out the [note section](https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-18048.html)
//! in the ELF specification.
//!
//! # Example
//!
//! ```
//! use noted::noted;
//! use goblin::Object;
//!
//! noted! {
//!     section = ".note.noted";
//!
//!     static FOO<"xxxxxxxx", 1>: [u8; 4] = [1, 2, 3, 4];
//!     static BAR<"yyyyy", 2>: u64 = 7;
//!     static BAZ<"zzz", 3>: u32 = 8;
//! }
//!
//! fn main() {
//!     // Load this binary
//!     let path = std::env::current_exe().unwrap();
//!     let buffer = std::fs::read(path).unwrap();
//!     let elf = match Object::parse(&buffer).unwrap() {
//!         Object::Elf(elf) => elf,
//!         _ => panic!("unsupported type"),
//!     };
//!
//!     // Parse and sort the notes in the specified section
//!     let mut notes: Vec<_> = elf
//!         .iter_note_sections(&buffer, Some(".note.noted"))
//!         .unwrap()
//!         .map(|x| x.unwrap())
//!         .collect();
//!     notes.sort_unstable_by_key(|x| x.n_type);
//!
//!     eprintln!("{:?}", notes);
//!
//!     // Validate all the notes
//!     assert_eq!(3, notes.len());
//!
//!     assert_eq!(notes[0].n_type, 1);
//!     assert_eq!(notes[1].n_type, 2);
//!     assert_eq!(notes[2].n_type, 3);
//!
//!     assert_eq!(notes[0].name, "xxxxxxxx");
//!     assert_eq!(notes[1].name, "yyyyy");
//!     assert_eq!(notes[2].name, "zzz");
//!
//!     assert_eq!(notes[0].desc, &[1, 2, 3, 4]);
//!     assert_eq!(notes[1].desc, &7u64.to_ne_bytes());
//!     assert_eq!(notes[2].desc, &8u32.to_ne_bytes());
//! }
//! ```

#![no_std]
#![deny(clippy::all)]
#![deny(missing_docs)]
#![allow(clippy::needless_doctest_main)]

#[repr(C, packed(4))]
struct Packed<T>(T);

#[repr(C, align(4))]
struct Aligned<T>(T);

/// A note as defined in the ELF specification
///
/// You probably don't want this struct. The `noted!` macro should provide
/// everything you need.
///
/// An instance of this struct should be binary compatible with notes as
/// defined in the ELF specification. However, you MUST put this note in
/// an appropriate ELF section. For example, `#[link_section = ".note"]`.
#[repr(C, align(4))]
pub struct Note<T, const N: usize> {
    namesz: u32,
    descsz: u32,
    kind: u32,
    name: [u8; N],
    desc: Aligned<Packed<T>>,
}

impl<T, const N: usize> Note<T, N> {
    /// Creates a new `Note` instance.
    ///
    /// You probably don't want this function. The `noted!` macro should
    /// provide everything you need.
    ///
    /// Note that if insufficient name bytes (i.e. `N`) are provided, the
    /// name will be silently truncated. You should use the provided macro
    /// (see above) to avoid this problem.
    pub const fn new(name: &'static str, id: u32, desc: T) -> Self {
        let mut buf = [0u8; N];

        let mut i = 0;
        while i < N - 1 {
            buf[i] = name.as_bytes()[i];
            i += 1;
        }

        Note {
            namesz: N as u32,
            descsz: core::mem::size_of::<T>() as u32,
            kind: id,
            name: buf,
            desc: Aligned(Packed(desc)),
        }
    }
}

/// A macro for creating ELF notes
///
/// See the module documentation for an example.
#[macro_export]
macro_rules! noted {
    (@internal $section:literal) => {};

    (
        @internal $section:literal

        $(#[$attr:meta])*
        $vis:vis static $symb:ident<$name:literal, $type:literal>: $kind:ty = $desc:expr;

        $($next:tt)*
    ) => {
        #[link_section = $section]
        $(#[$attr])*
        #[used]
        $vis static $symb: $crate::Note<$kind, {$name.len() + 1}> = $crate::Note::new($name, $type, $desc);

        noted! { @internal $section $($next)* }
    };

    (section = $section:literal; $($next:tt)+) => {
        noted! { @internal $section $($next)+ }
    };

    ($($next:tt)+) => {
        noted! { @internal ".note" $($next)+ }
    };
}
