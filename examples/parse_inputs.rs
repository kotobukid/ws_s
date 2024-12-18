use std::iter::Peekable;
use std::str::Chars;

fn parse_arguments(input: &str) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    let mut current_arg: String = String::new();
    let mut chars: Peekable<Chars> = input.chars().peekable();
    let mut in_quotes: bool = false;
    let mut quote_char: char = '\0';

    while let Some(&c) = chars.peek() {
        match c {
            // クォートの開始もしくは終了
            '\'' | '"' => {
                if in_quotes && c == quote_char {
                    // クォート終了
                    in_quotes = false;
                    chars.next(); // カーソルを次に進める
                } else if !in_quotes {
                    // クォート開始
                    in_quotes = true;
                    quote_char = c;
                    chars.next(); // カーソルを次に進める
                } else {
                    // クォート内の通常の文字を追加
                    current_arg.push(c);
                    chars.next(); // カーソルを次に進める
                }
            }
            ' ' if !in_quotes => {
                // スペースが出現した場合、クォートの外なら現在の引数を終了
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
                chars.next(); // スペースをスキップ
            }
            _ => {
                // 通常の文字を追加
                current_arg.push(c);
                chars.next(); // カーソルを次に進める
            }
        }
    }

    // 最後の引数を追加
    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input: &str) -> String {
    let mut output = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        match c {
            '\'' | '"' => {
                if in_quotes && c == quote_char {
                    in_quotes = false;

                    // この関数はクオートを置換したりしない
                    output.push(c);
                } else if !in_quotes {
                    in_quotes = true;
                    quote_char = c;

                    // 同上
                    output.push(c);
                } else {
                    output.push(c);
                }
            }
            '　' => {
                // 全角スペース
                if !in_quotes {
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

    let args1 = parse_arguments(replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1).as_str());
    let args2 = parse_arguments(replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input2).as_str());

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

        assert_eq!(args1, vec!["/name", "taro suzuki", "age", "25"]);
        assert_eq!(args1a, vec!["/name", "taro s'uzuki", "age", "25"]);
        assert_eq!(args1b, vec!["/name", "taro　suzuki", "age", "25"]);
        assert_eq!(args2, vec!["/message", "Hello, how are you?"]);
        assert_eq!(
            args3,
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
            args4,
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
}
