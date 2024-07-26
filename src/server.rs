use mlua::{Function, Lua};
use std::{fs, rc::Rc, sync::Arc, time::Duration};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    runtime::Builder,
    task, time,
};
use tracing::{error, info};

use crate::{
    cli::ServeArgs,
    script::{self, Ctx},
};

pub async fn serve(args: &ServeArgs) -> anyhow::Result<()> {
    let listener = TcpListener::bind(&args.listen).await?;
    info!("listening on: {}", args.listen);
    info!("backend: {}", args.backend);

    let buf_size = args.buf_size;

    let rt = Arc::new(Builder::new_multi_thread().enable_all().build()?);

    loop {
        let (mut client_stream, _) = listener.accept().await?;
        let backend_stream = match TcpStream::connect(&args.backend).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("connect to backend error: {}", e);
                client_stream.shutdown().await?;
                continue;
            }
        };

        let ctx = Ctx {
            buf_size,
            client_stream,
            backend_stream,
        };

        let args = args.clone();
        let rt = rt.clone();

        task::spawn(async move {
            let _ = task::spawn_blocking(move || rt.block_on(handle(args, ctx))).await;
        });
    }
}

async fn handle(args: ServeArgs, ctx: Ctx) -> anyhow::Result<()> {
    let lua = Rc::new(Lua::new());

    script::bind(lua.clone(), &args)?;
    script::load(lua.clone(), &args.script)?;

    let globals = lua.globals();
    let handle: Function = globals.get(args.script_entry.clone())?;

    let call = async {
        info!("calling script entry: {}", args.script_entry);
        match handle.call_async::<_, ()>(ctx).await {
            Ok(_) => {}
            Err(e) => {
                error!("handle error: {}", e);
            }
        }
    };

    let mut interval = time::interval(Duration::from_secs(10));
    let mut last_time = None;

    let lua = lua.clone();

    let watch = async {
        loop {
            interval.tick().await;
            if let Ok(meta) = fs::metadata(&args.script) {
                let modified = Some(meta.modified().unwrap());
                if last_time.is_none() {
                    last_time = modified;
                } else if modified > last_time {
                    info!("script file changed, reload");
                    last_time = modified;
                    if let Err(e) = script::load(lua.clone(), &args.script) {
                        error!("reload script error: {}", e);
                    }
                }
            }
        }
    };

    tokio::select! {
        _ = call => {}
        _ = watch => {}
    }

    Ok(())
}
