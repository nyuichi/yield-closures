# yield-closures [![Crates.io](https://img.shields.io/crates/v/yield-closures)](https://crates.io/crates/yield-closures) [![docs.rs](https://img.shields.io/docsrs/yield-closures)](https://docs.rs/yield-closures/)

An implementation of [MCP-49](https://github.com/rust-lang/lang-team/issues/49).

```rust
#[test]
fn decode_escape_string() {
    let escaped_text = "Hello,\x20world!\\n";
    let text: String = escaped_text
        .chars()
        .filter_map(co!(|c| {
            loop {
                if c != '\\' {
                    // Not escaped
                    yield Some(c);
                    continue;
                }

                // Go past the \
                yield None;

                // Unescaped-char
                match c {
                    // Hexadecimal
                    'x' => {
                        yield None; // Go past the x
                        let most = c.to_digit(16);
                        yield None; // Go past the first digit
                        let least = c.to_digit(16);
                        // Yield the decoded char if valid
                        yield (|| char::from_u32(most? << 4 | least?))()
                    }
                    // Simple escapes
                    'n' => yield Some('\n'),
                    'r' => yield Some('\r'),
                    't' => yield Some('\t'),
                    '0' => yield Some('\0'),
                    '\\' => yield Some('\\'),
                    // Unnecessary escape
                    _ => yield Some(c),
                }
            }
        }))
        .collect();
    assert_eq!(text, "Hello, world!\n");
}
```

For the details of the proposal, see https://lang-team.rust-lang.org/design_notes/general_coroutines.html.

Differences between this implementation and the proposal are summarized below:

- This crate offers a macro implementation. It works with the stable Rust.
- No `FnPin` is provided. Yield closures made with this crate use `Box::pin` internally and hence `FnMut`.
- In yield closures, one cannot use `return` expressions.
- The body of a yield closure must be explosive i.e. must not return and typed by the `!` type. Thus it is compatible with both of the two designs of yield closures discussed in the document of MCP-49: poisoning by default or not.
