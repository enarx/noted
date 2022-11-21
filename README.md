[![Workflow Status](https://github.com/enarx/noted/workflows/test/badge.svg)](https://github.com/enarx/noted/actions?query=workflow%3A%22test%22)
[![Average time to resolve an issue](https://isitmaintained.com/badge/resolution/enarx/noted.svg)](https://isitmaintained.com/project/enarx/noted "Average time to resolve an issue")
[![Percentage of issues still open](https://isitmaintained.com/badge/open/enarx/noted.svg)](https://isitmaintained.com/project/enarx/noted "Percentage of issues still open")
![Maintenance](https://img.shields.io/badge/maintenance-activly--developed-brightgreen.svg)

# noted

This crate implements a macro that defines ELF notes for communicating
information to the tooling that processes ELF notes.

Be sure to check out the [note section](https://docs.oracle.com/cd/E23824_01/html/819-0690/chapter6-18048.html)
in the ELF specification.

## Example

```rust
use noted::noted;
use goblin::Object;

const YYYYY: &str = "yyyyy";
const TWO: u32 = 2;

noted! {
    section = ".note.noted";

    static FOO<"xxxxxxxx", 1, [u8; 4]> = [1, 2, 3, 4];
    static BAR<YYYYY, TWO, u64> = 7;
    static BAZ<"zzz", 3, u32> = 8;
}

fn main() {
    // Load this binary
    let path = std::env::current_exe().unwrap();
    let buffer = std::fs::read(path).unwrap();
    let elf = match Object::parse(&buffer).unwrap() {
        Object::Elf(elf) => elf,
        _ => panic!("unsupported type"),
    };

    // Parse and sort the notes in the specified section
    let mut notes: Vec<_> = elf
        .iter_note_sections(&buffer, Some(".note.noted"))
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    notes.sort_unstable_by_key(|x| x.n_type);

    eprintln!("{:?}", notes);

    // Validate all the notes
    assert_eq!(3, notes.len());

    assert_eq!(notes[0].n_type, 1);
    assert_eq!(notes[1].n_type, TWO);
    assert_eq!(notes[2].n_type, 3);

    assert_eq!(notes[0].name, "xxxxxxxx");
    assert_eq!(notes[1].name, YYYYY);
    assert_eq!(notes[2].name, "zzz");

    assert_eq!(notes[0].desc, &[1, 2, 3, 4]);
    assert_eq!(notes[1].desc, &7u64.to_ne_bytes());
    assert_eq!(notes[2].desc, &8u32.to_ne_bytes());
}
```

License: Apache-2.0
