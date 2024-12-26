use crossterm::terminal::ClearType;
use crossterm::{
    event::{Event, KeyCode},
    terminal::{self},
    ExecutableCommand,
};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // ログ初期化
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    // 標準入力が接続されていない場合終了
    if !atty::is(atty::Stream::Stdin) {
        eprintln!("このプログラムは標準入力が必要です。");
        return Ok(());
    }

    // 選択肢
    let options = vec!["Javascript", "Typescript", "Abort"];
    let mut selected = 0;

    // ターミナルを RawMode に設定
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    let prompt = Some("Select Language.");

    let mut current_pos: (u16, u16) = crossterm::cursor::position()?;

    // 初期表示
    print_menu(&mut stdout, current_pos, prompt, &options, selected)?;

    loop {
        // 入力待機 (タイムアウトを設定)
        if crossterm::event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(event) = crossterm::event::read()? {
                if event.kind == crossterm::event::KeyEventKind::Press {
                    match event.code {
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if selected < options.len() - 1 {
                                selected += 1;
                            }
                        }
                        KeyCode::Enter => {
                            terminal::disable_raw_mode()?;
                            stdout.execute(crossterm::cursor::MoveDown(1))?;
                            if selected == options.len() - 1 {
                                println!("中断されました");
                                break;
                            } else {
                                println!("選択された項目: {}", options[selected]);
                                break;
                            }
                        }
                        KeyCode::Char('c') | KeyCode::Esc => {
                            terminal::disable_raw_mode()?;
                            println!("中断されました");
                            break;
                        }
                        _ => {}
                    }
                    print_menu(&mut stdout, current_pos, prompt, &options, selected)?;
                }
            }
            current_pos = crossterm::cursor::position()?;
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

#[allow(unused_variables)]
fn print_menu(
    stdout: &mut io::Stdout,
    start_pos: (u16, u16),
    prompt: Option<&str>,
    options: &[&str],
    selected: usize,
) -> io::Result<()> {
    // カーソルを初期の表示位置に移動
    stdout
        .execute(crossterm::cursor::MoveTo(0, 0))?
        .execute(terminal::Clear(ClearType::FromCursorDown))?;

    // プロンプトを表示
    if let Some(prompt) = prompt {
        writeln!(stdout, "{}", prompt)?;
    }

    // 選択肢を表示
    for (i, option) in options.iter().enumerate() {
        if i == selected {
            writeln!(stdout, "> {}", option)?;
        } else {
            writeln!(stdout, "  {}", option)?;
        }
    }

    stdout.flush()?;
    Ok(())
}
