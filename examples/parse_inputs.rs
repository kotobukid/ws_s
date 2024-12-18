use std::iter::Peekable;
use std::str::Chars;

fn parse_arguments(input: &str) -> Result<Vec<String>, String> {
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

fn replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_quotes: Option<char> = None;
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => {
                if Some(c) == in_quotes {
                    in_quotes = None;

                    // この関数はクオートを置換したりしない
                    output.push(c);
                } else if in_quotes.is_none() {
                    in_quotes = Some(c);
                    // 同上
                    output.push(c);
                } else {
                    output.push(c);
                }
            }
            '　' => {
                // 全角スペース
                if in_quotes.is_none() {
                    output.push(' ');
                } else {
                    output.push(c);
                }
            }
            _ => {
                output.push(c);
            }
        }
    }

    output
}

fn main() {
    let input1 = "/name \"taro suzuki\" age 25";
    let input2 = "/message 'Hello, how are you?'";

    let args1 = parse_arguments(
        replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1).as_str(),
    );
    let args2 = parse_arguments(
        replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input2).as_str(),
    );

    println!("{:?}", args1); // ["/name", "taro suzuki", "age", "25"]
    println!("{:?}", args2); // ["/message", "Hello, how are you?"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes() {
        let input1 = "/name \"taro suzuki\" age 25";
        let input2 = "/message 'Hello, how are you?' age　20";

        let output1 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1);
        let output2 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input2);

        assert_eq!(output1, "/name \"taro suzuki\" age 25");
        assert_eq!(output2, "/message 'Hello, how are you?' age 20");
    }

    #[test]
    fn test_parse_arguments() {
        let input1 = "/name \"taro suzuki\" age 25";
        let input1a = "/name \"taro s'uzuki\" age 25";
        let input1b = "/name \"taro　suzuki\" age　25"; // 全角スペース2箇所
        let input2 = "/message 'Hello, how are you?'";
        let input3 = "/message 'Hello, how are you?' /name \"taro suzuki\" age 25";
        let input4 = "/message 'Hello, how are you?' /name \"taro suzuki\" age 25 /message 'Hello, how are you?'";

        let args1 = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1).as_str(),
        );
        let args1a = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1a).as_str(),
        );
        let args1b = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1b).as_str(),
        );
        let args2 = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input2).as_str(),
        );
        let args3 = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input3).as_str(),
        );
        let args4 = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input4).as_str(),
        );

        assert_eq!(args1.unwrap(), vec!["/name", "taro suzuki", "age", "25"]);
        assert_eq!(args1a.unwrap(), vec!["/name", "taro s'uzuki", "age", "25"]);
        assert_eq!(args1b.unwrap(), vec!["/name", "taro　suzuki", "age", "25"]);
        assert_eq!(args2.unwrap(), vec!["/message", "Hello, how are you?"]);
        assert_eq!(
            args3.unwrap(),
            vec![
                "/message",
                "Hello, how are you?",
                "/name",
                "taro suzuki",
                "age",
                "25"
            ]
        );
        assert_eq!(
            args4.unwrap(),
            vec![
                "/message",
                "Hello, how are you?",
                "/name",
                "taro suzuki",
                "age",
                "25",
                "/message",
                "Hello, how are you?"
            ]
        );
    }

    #[test]
    fn test_parse_arguments_with_error_expectation() {
        let input = "/name \"taro suzuki age 25"; // クォートが閉じられていない入力

        // パース関数を実行
        let result = parse_arguments(
            replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input).as_str(),
        );

        // エラーが発生することを確認
        assert!(
            result.is_err(),
            "result should be an error for unmatched quotes"
        );

        // エラー内容が正しいか確認
        if let Err(err) = result {
            assert_eq!(err, "クォートが閉じられていません");
        } else {
            panic!("Expected an error but got Ok");
        }
    }
}
