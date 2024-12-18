use std::iter::Peekable;
use std::str::Chars;

pub fn parse_arguments(input: &str) -> Result<Vec<String>, String> {
    let mut args: Vec<String> = Vec::new();
    let mut current_arg: String = String::with_capacity(input.len());
    let mut chars: Peekable<Chars> = input.chars().peekable();
    let mut in_quotes: Option<char> = None;

    while let Some(&c) = chars.peek() {
        match c {
            // クォートの開始もしくは終了
            '\'' | '"' => {
                if let Some(q) = in_quotes {
                    if c == q {
                        // クォート終了
                        in_quotes = None;
                        chars.next(); // カーソルを次に進める
                    } else {
                        current_arg.push(c);
                        chars.next();
                    }
                } else {
                    // クォート開始
                    in_quotes = Some(c);
                    chars.next(); // カーソルを次に進める
                }
            }
            ' ' if in_quotes.is_none() => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
                chars.next();
            }
            _ => {
                // 通常の文字を追加
                current_arg.push(c);
                chars.next(); // カーソルを次に進める
            }
        }
    }

    if in_quotes.is_some() {
        return Err("クォートが閉じられていません".to_string());
    }

    // 最後の引数を追加
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    Ok(args)
}

pub fn replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_quotes: Option<char> = None;
    let mut chars = input.chars();
    let mut last_was_space = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => {
                if Some(c) == in_quotes {
                    in_quotes = None;
                    output.push(c);
                } else if in_quotes.is_none() {
                    in_quotes = Some(c);
                    output.push(c);
                } else {
                    output.push(c);
                }
                last_was_space = false;
            }
            '　' => {
                if in_quotes.is_none() {
                    if !last_was_space && !output.is_empty() {
                        output.push(' ');
                        last_was_space = true;
                    }
                } else {
                    output.push(c);
                }
            }
            ' ' => {
                if in_quotes.is_none() {
                    if !last_was_space && !output.is_empty() {
                        output.push(c);
                        last_was_space = true;
                    }
                } else {
                    output.push(c);
                }
            }
            _ => {
                output.push(c);
                last_was_space = false;
            }
        }
    }

    // ここで末尾の不要なスペースを削除する
    if in_quotes.is_none() {
        output = output.trim_end().to_string();
    }

    output
}
