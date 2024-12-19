use std::io::{self, Write};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    cursor,
    ExecutableCommand,
    QueueableCommand,
};

fn main() -> io::Result<()> {
    // 選択肢
    let options = vec!["Javascript", "Typescript"];
    let mut selected = 0;

    // ターミナルを Raw Mode に設定
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // 初期表示
    print_menu(&mut stdout, &options, selected)?;

    loop {
        if let Event::Key(event) = event::read()? {
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
                    // 選択完了
                    terminal::disable_raw_mode()?;
                    stdout.execute(cursor::MoveDown(1))?;
                    println!("選択された項目: {}", options[selected]);
                    break;
                }
                KeyCode::Char('c') if event.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    // Ctrl+C で終了
                    terminal::disable_raw_mode()?;
                    stdout.execute(cursor::MoveDown(1))?;
                    println!("中断されました");
                    break;
                }
                _ => {}
            }

            print_menu(&mut stdout, &options, selected)?;
        }
    }

    Ok(())
}

fn print_menu(stdout: &mut io::Stdout, options: &[&str], selected: usize) -> io::Result<()> {
    // カーソルを先頭に移動して画面をクリア
    stdout
        .queue(cursor::SavePosition)?
        .queue(cursor::MoveTo(0, 0))?;

    for (i, option) in options.iter().enumerate() {
        stdout
            .queue(terminal::Clear(ClearType::CurrentLine))?;

        if i == selected {
            println!("\r> {}", option);
        } else {
            println!("\r  {}", option);
        }
    }

    stdout
        .queue(cursor::RestorePosition)?
        .flush()?;

    Ok(())
}