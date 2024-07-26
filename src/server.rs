use mlua::{Function, Lua};
use std::{rc::Rc, sync::Arc};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    runtime::Builder,
    task,
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

    script::build_script(lua.clone(), &args)?;

    let globals = lua.globals();
    let handle: Function = globals.get(args.script_entry.clone())?;

    match handle.call_async::<_, ()>(ctx).await {
        Ok(_) => {}
        Err(e) => {
            error!("handle error: {}", e);
        }
    }

    Ok(())
}
