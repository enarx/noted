// SPDX-License-Identifier: Apache-2.0

#![doc = include_str!("../README.md")]
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
        $vis:vis static $symb:ident<$name:expr, $type:expr, $kind:ty> = $desc:expr;

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
