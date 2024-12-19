use std::io::{self, Write};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    cursor,
    ExecutableCommand,
    QueueableCommand,
};

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;


fn main() -> io::Result<()> {
    // ログ初期化
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    // 確認用ログ
    info!("プログラムが開始されました");

    // 選択肢
    let options = vec!["Javascript", "Typescript"];
    let mut selected = 0;

    // ターミナルを Raw Mode に設定
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();

    // 初期表示
    print_menu(&mut stdout, &options, selected)?;

    // 標準入力が接続されていない場合は終了させる
    if !atty::is(atty::Stream::Stdin) {
        eprintln!("このプログラムは標準入力が必要です。");
        return Ok(());
    } else {
        println!("標準入力を使用可能です。");
    }

    // イベントループ中にログを追加します（下記参照）
    loop {
        if let Ok(ev) = event::read() {
            info!("Received event: {:?}", ev);
            match ev {
                Event::Key(key_event) => {
                    info!("キーイベント: {:?}", key_event);
                    // 残りの処理...
                    if key_event.code == KeyCode::Char('c') {
                        terminal::disable_raw_mode()?;
                        println!("中断されました");
                        break;
                    }
                },
                _ => {}
            }
        } else {
            info!("Event reading failed or returned null");
        }
    }

    terminal::disable_raw_mode()?;
    Ok(())
}

fn print_menu(stdout: &mut io::Stdout, options: &[&str], selected: usize) -> io::Result<()> {
    // 画面全体をクリア
    stdout
        .execute(terminal::Clear(ClearType::All))?
        .execute(crossterm::cursor::MoveTo(0, 0))?;

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