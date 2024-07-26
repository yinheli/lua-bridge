use mlua::{Function, Lua};
use std::rc::Rc;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    task,
};
use tracing::{error, info};

use crate::{
    cli::ServeArgs,
    script::{self, Ctx},
};

pub async fn serve(args: &ServeArgs) -> anyhow::Result<()> {
    let lua = Rc::new(Lua::new());

    script::build_script(lua.clone(), args)?;

    let globals = lua.globals();
    let handle: Function = globals.get(args.script_entry.clone())?;
    let handle = Rc::new(handle);

    let listener = TcpListener::bind(&args.listen).await?;
    info!("listening on: {}", args.listen);
    info!("backend: {}", args.backend);
    loop {
        let (mut request_stream, _) = listener.accept().await?;
        let backend_stream = match TcpStream::connect(&args.backend).await {
            Ok(stream) => stream,
            Err(e) => {
                error!("connect to backend error: {}", e);
                request_stream.shutdown().await?;
                continue;
            }
        };
        let handle = handle.clone();

        task::LocalSet::new()
            .run_until(async move {
                let ctx = Ctx {
                    buf_size: args.buf_size,
                    client_stream: request_stream,
                    backend_stream,
                };
                match handle.call_async::<_, ()>(ctx).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("handle error: {}", e);
                    }
                }
            })
            .await;
    }
}
