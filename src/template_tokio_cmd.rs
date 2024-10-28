#![allow(unused_imports)]
#![allow(dead_code)]
use std::{
    env,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    net::TcpStream,
    process::Command,
    signal,
    sync::{mpsc, oneshot},
    task,
};

// TODO::refactor - proper struct and traits

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Please provide a command to run");
        std::process::exit(1);
    }

    println!("shelly-run -> {:?}", args);
    let commands: Vec<Vec<String>> = args
        .split(|arg| arg == "--")
        .map(|arg| arg.to_vec())
        .collect();

    let (io_tx, mut io_rx) = mpsc::channel::<String>(1000);
    let mut vec_oneshot_rx = Vec::new();

    let mut handles = Vec::new();

    // //? configure what and who the runner knows..
    // if "--run-config" == args[0] {
    //     args.remove(0); // remove the --run-config flag
    //     let prog = args.remove(0);

    //     Command::new(prog)
    //         .args(args)
    //         .spawn()
    //         .expect("Runner configuration");
    // }
    for command in commands.into_iter() {
        if command.is_empty() {
            continue;
        }

        let io_tx = io_tx.clone();
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        vec_oneshot_rx.push(oneshot_rx);

        let handle = task::spawn(async move {
            parse_and_run(command, io_tx, oneshot_tx).await;
        });

        handles.push(handle);
    }

    //? recv -> mpsc io, currently stdout receiver
    task::spawn(async move {
        while let Some(line) = io_rx.recv().await {
            println!("{}", line);
        }
    });

    task::spawn(async move {
        for handle in handles {
            let _ = handle.await;
        }
        println!("pocess handles done");
    });

    for oneshot_rx in vec_oneshot_rx {
        let _ = oneshot_rx.await;
    }
    println!("oneshots done - exit");
}

async fn exec(
    cmd: String,
    args: Vec<String>,
    io_tx: mpsc::Sender<String>,
    oneshot_tx: oneshot::Sender<()>,
) {
    //? hello little one
    println!("exec - {}", cmd);
    let mut child = Command::new(cmd.clone())
        .kill_on_drop(true)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run callback command");

    let cleanup_io_tx = io_tx.clone();
    let io_tx_stdout = io_tx.clone();
    let io_tx_stderr = io_tx.clone();

    // Handle stdout
    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if io_tx_stdout.send(line).await.is_err() {
                    break;
                }
            }
        });
    }

    // Handle stderr
    if let Some(stderr) = child.stderr.take() {
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if io_tx_stderr.send(line).await.is_err() {
                    break;
                }
            }
        });
    }

    let _ = cleanup_io_tx.send("hi".to_string());
    //TODO::pocesses - proper greeting, id's & metadata
    #[cfg(unix)]
    tokio::select! {
        _ = if let Some(mut sigterm) = sigterm {
            sigterm.recv().await
        }  => {
            println!("exec sigkill {:?} .. __unix__", child);
            cleanup.await;
        },
        status = child.wait() => {
            let stat = status;
            println!("exec ok {:?}", stat);

            let _ = oneshot_tx.send(());
        },
        // _ = &mut oneshot_rx => {
        //     child.kill().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        // }
    }

    #[cfg(not(unix))]
    tokio::select! {
        status = child.wait() => {
            let stat = status;
            println!("exec ok {:?}", stat);

            let _ = oneshot_tx.send(());
        },
        _ = signal::ctrl_c() => {
            let _ = child.kill().await;
            let _ = oneshot_tx.send(());
            let _ = cleanup_io_tx.send("bye".to_string());

            println!("exec sigkill {:?}", child);
        }
        // _ = &mut oneshot_rx => {
        //     child.kill().await.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        // }
    }

    //? goodbye little one
    let _ = cleanup_io_tx.send("hi".to_string());
}

async fn parse_and_run(
    command: Vec<String>,
    io_tx: mpsc::Sender<String>,
    coneshot_tx: oneshot::Sender<()>,
) {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    let mut task_args: Vec<Vec<String>> = command
        .split(|arg| arg == "--cb")
        .map(|arg| arg.to_vec())
        .collect();

    let mut main_args = task_args.remove(0);
    let prog = main_args.remove(0);
    let io_tx = io_tx.clone();

    exec(prog, main_args, io_tx.clone(), oneshot_tx).await;

    println!("prep callback {:?}", task_args);
    if !task_args.is_empty() {
        let mut callback_args = task_args.remove(0);
        println!("- callback: {callback_args:?}");

        if !callback_args.is_empty() {
            let callback_prog = callback_args.remove(0);

            let _ = oneshot_rx.await;

            exec(callback_prog, callback_args, io_tx.clone(), coneshot_tx).await;
        }
    }

    println!("\nreturning from run {command:?}\n\n");
    println!(" ");
}

// async fn tcp_mock(command: Vec<String>, mut stream: TcpStream) {
//     // Simulate command execution and sending output over TCP
//     let output = format!("Executed command: {:?}", command);
//     let _ = stream.write_all(output.as_bytes()).await;
//     // Close the connection to signal completion
//     let _ = stream.shutdown().await;
// }

// #[tokio::main]
// async fn db_handler() {
//     let args: Vec<String> = env::args().skip(1).collect();
//     if args.is_empty() {
//         eprintln!("Please provide a command to run");
//         process::exit(1);
//     }

//     println!("shelly-run -> {:?}", args);
//     let commands: Vec<Vec<String>> = args
//         .split(|arg| arg == "--")
//         .map(|arg| arg.to_vec())
//         .collect();

//     let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
//     let addr = listener.local_addr().unwrap();
//     println!("Listening on: {}", addr);

//     let server = task::spawn(async move {
//         loop {
//             let (socket, _) = listener.accept().await.unwrap();
//             task::spawn(async move {
//                 let mut buffer = [0; 1024];
//                 let mut stream = socket;
//                 let n = stream.read(&mut buffer).await.unwrap();
//                 if n == 0 {
//                     return;
//                 }
//                 println!("{}", String::from_utf8_lossy(&buffer[..n]));
//             });
//         }
//     });

//     let mut handles = Vec::new();
//     for command in commands.into_iter() {
//         if command.is_empty() {
//             continue;
//         }

//         // let (oneshot_tx, oneshot_rx) = oneshot::channel::<()>();
//         let handle = task::spawn(async move {
//             let stream = TcpStream::connect(addr).await.unwrap();
//             tcp_mock(command, stream).await;
//         });

//         handles.push(handle);
//     }

//     for handle in handles {
//         let _ = handle.await;
//     }

//     // Wait for the server task to complete, which in a real application might be signaled to shut down gracefully
//     let _ = server.await;
// }
