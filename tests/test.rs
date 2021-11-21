use yield_closures::co;

#[test]
fn echo() {
    let mut f = co!(|x| {
        loop {
            yield x;
        }
    });
    assert_eq!(f(1), 1);
    assert_eq!(f(2), 2);
    assert_eq!(f(3), 3);
}

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

#[test]
fn fib() {
    let mut f = co!(|| {
        let (mut x, mut y) = (1, 1);
        loop {
            yield x;
            let z = x;
            x = y;
            y += z;
        }
    });
    assert_eq!(f(), 1);
    assert_eq!(f(), 1);
    assert_eq!(f(), 2);
    assert_eq!(f(), 3);
    assert_eq!(f(), 5);
}

#[test]
fn decode_base64() {
    let s = "QUJDREVGRw";
    let char_to_sextet = |c: char| {
        if ('A'..='Z').contains(&c) {
            c as u8 - b'A'
        } else if ('a'..='z').contains(&c) {
            26 + c as u8 - b'a'
        } else if ('0'..='9').contains(&c) {
            52 + c as u8 - b'0'
        } else if c == '+' {
            62
        } else if c == '/' {
            63
        } else {
            panic!("invalid as base64");
        }
    };
    let mut output = vec![];
    s.chars().for_each(co!(|x| loop {
        let a = char_to_sextet(x);
        yield;
        let b = char_to_sextet(x);
        output.push(a << 2 | b >> 4); // aaaaaabb
        yield;
        let c = char_to_sextet(x);
        output.push((b & 0b1111) << 4 | c >> 2); // bbbbcccc
        yield;
        output.push((c & 0b11) << 6 | char_to_sextet(x)); // ccdddddd
        yield;
    }));
    assert_eq!(String::from_utf8_lossy(&output), "ABCDEFG");
}

#[test]
fn game() {
    #[derive(PartialEq, Debug, Copy, Clone)]
    enum Action {
        Wander,
        Attack,
        Evade,
        Heal,
    }
    use Action::*;

    let mut game = co!(|is_opponent_near: bool, my_health: u32| -> Action {
        loop {
            // Find opponent
            while !is_opponent_near {
                yield Wander;
            }

            // Do battle!
            let mut min_health = my_health;
            while my_health > 1 && is_opponent_near {
                yield Attack;
                if my_health < min_health {
                    min_health = my_health;
                    yield Evade;
                }
            }

            // Recover
            if my_health < 5 {
                yield Heal;
            }
        }
    });

    assert_eq!(game(false, 3), Wander);
    assert_eq!(game(true, 3), Attack);
    assert_eq!(game(true, 2), Evade);
    assert_eq!(game(true, 2), Attack);
    assert_eq!(game(true, 1), Evade);
    assert_eq!(game(true, 1), Heal);
    assert_eq!(game(true, 5), Attack);
    assert_eq!(game(false, 5), Wander);
}
