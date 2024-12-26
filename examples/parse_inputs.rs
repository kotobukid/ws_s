use ws_s::utils::{
    parse_arguments, replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes,
};

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

#[test]
fn test_replace_full_width_spaces_with_leading_spaces() {
    let input1 = "    first second";
    let input2 = "　　first　second　third";
    let input3 = "    'quoted   string'   ";
    let input4 = " 　"; // スペースだけの入力

    let output1 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input1);
    let output2 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input2);
    let output3 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input3);
    let output4 = replace_full_width_spaces_to_half_width_spaces_if_not_in_quotes(input4);

    assert_eq!(output1, "first second");
    assert_eq!(output2, "first second third");
    assert_eq!(output3, "'quoted   string'");
    assert_eq!(output4, ""); // スペースだけの入力は空文字になる
}
