use lazy_static::lazy_static;
use mlua::{Lua, UserData};
use r2d2::Pool;
use r2d2_mysql::MySqlConnectionManager;
use std::{
    fs,
    io::ErrorKind::{BrokenPipe, NotConnected},
    ops::Deref,
    rc::Rc,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{error, info, warn};

use crate::cli::ServeArgs;

pub fn build_script(lua: Rc<Lua>, args: &ServeArgs) -> anyhow::Result<()> {
    let globals = lua.globals();
    let id = format!("{:p}", lua.deref());
    globals.set("_instance_id", id)?;
    globals.set("_version", env!("CARGO_PKG_VERSION"))?;

    register_log_function(lua.clone())?;
    regisger_help_function(lua.clone())?;
    regisger_mysql_function(lua.clone(), &args.mysql_uri)?;
    regisger_redis_function(lua.clone(), &args.redis_uri)?;

    let script = fs::read_to_string(&args.script)?;
    lua.clone().load(script).exec()?;

    Ok(())
}

fn register_log_function(lua: Rc<Lua>) -> anyhow::Result<()> {
    let globals = lua.globals();

    let info = lua.create_function(|_, arg: String| {
        info!("[scripts] {}", arg.trim_end());
        Ok(())
    })?;
    globals.set("info", info)?;

    let error = lua.create_function(|_, arg: String| {
        error!("[scripts] {}", arg.trim_end());
        Ok(())
    })?;
    globals.set("error", error)?;

    let warn = lua.create_function(|_, arg: String| {
        warn!("[scripts] {}", arg.trim_end());
        Ok(())
    })?;
    globals.set("warn", warn)?;

    Ok(())
}

fn regisger_help_function(lua: Rc<Lua>) -> anyhow::Result<()> {
    let globals = lua.globals();

    let bytes_to_string = lua.create_function(|_, arg: Vec<u8>| {
        if arg.is_empty() {
            return Ok("".to_string());
        }
        match String::from_utf8(arg) {
            Ok(s) => Ok(s),
            Err(e) => Ok(format!("{:?}", e)),
        }
    })?;
    globals.set("bytes_to_string", bytes_to_string)?;

    Ok(())
}

fn regisger_mysql_function(lua: Rc<Lua>, uri: &str) -> anyhow::Result<()> {
    if uri.is_empty() {
        info!("mysql uri is empty, skip register mysql functions");
        return Ok(());
    }
    std::env::set_var("_mysql_uri", uri);
    lazy_static! {
        static ref POOL: Arc<Pool<MySqlConnectionManager>> = {
            let uri = std::env::var("_mysql_uri").unwrap();
            let opts = r2d2_mysql::mysql::Opts::from_url(&uri).unwrap();
            let builder = r2d2_mysql::mysql::OptsBuilder::from_opts(opts);
            let manager = MySqlConnectionManager::new(builder);
            let pool = Arc::new(Pool::builder().build(manager).unwrap());
            std::env::remove_var("_mysql_uri");
            pool
        };
    };

    let globals = lua.globals();

    // add functions for mysql

    let query = lua.create_function(|_, _sql: String| {
        let pool = POOL.clone();
        let _conn = pool.get().unwrap();

        // TODO

        Ok(())
    })?;
    globals.set("query", query)?;

    Ok(())
}

fn regisger_redis_function(lua: Rc<Lua>, uri: &str) -> anyhow::Result<()> {
    if uri.is_empty() {
        info!("redis uri is empty, skip register redis functions");
        return Ok(());
    }
    std::env::set_var("_redis_uri", uri);
    lazy_static! {
        static ref POOL: Arc<Pool<redis::Client>> = {
            let uri = std::env::var("_redis_uri").unwrap();
            let client = redis::Client::open(uri).unwrap();
            let pool = Arc::new(r2d2::Pool::builder().build(client).unwrap());
            std::env::remove_var("_redis_uri");
            pool
        };
    };

    let globals = lua.globals();

    // add functions for redis

    let get = lua.create_function(|_, _key: String| {
        let pool = POOL.clone();
        let _conn = pool.get().unwrap();

        // TODO

        Ok(())
    })?;
    globals.set("get", get)?;

    Ok(())
}

pub struct Ctx {
    pub buf_size: usize,
    pub client_stream: TcpStream,
    pub backend_stream: TcpStream,
}

impl Ctx {
    async fn read_stream(
        stream: &mut TcpStream,
        size: usize,
        buf_size: usize,
    ) -> Result<Vec<u8>, std::io::Error> {
        let size = if size > 0 { size } else { buf_size };
        let mut buf = vec![0; size];
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            stream.shutdown().await?;
        }
        Ok(buf[..n].to_vec())
    }

    async fn write_stream(stream: &mut TcpStream, data: &[u8]) -> Result<usize, std::io::Error> {
        match stream.write(data).await {
            Ok(n) => Ok(n),
            Err(e) => {
                if e.kind() == BrokenPipe || e.kind() == NotConnected {
                    let _ = stream.shutdown().await;
                    return Ok(0);
                }
                error!("write error: {}", e);
                Ok(0)
            }
        }
    }

    async fn close_stream(stream: &mut TcpStream) -> Result<(), std::io::Error> {
        let _ = stream.shutdown().await;
        Ok(())
    }

    fn to_lua_result<T>(result: Result<T, std::io::Error>) -> mlua::Result<T> {
        result.map_err(mlua::Error::external)
    }
}

impl UserData for Ctx {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(_fields: &mut F) {}

    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method_mut("read_client", |_, this, size: usize| async move {
            Self::to_lua_result(
                Self::read_stream(&mut this.client_stream, size, this.buf_size).await,
            )
        });

        methods.add_async_method_mut("write_client", |_, this, data: Vec<u8>| async move {
            Self::to_lua_result(Self::write_stream(&mut this.client_stream, &data).await)
        });

        methods.add_async_method_mut("read_backend", |_, this, size: usize| async move {
            Self::to_lua_result(
                Self::read_stream(&mut this.backend_stream, size, this.buf_size).await,
            )
        });

        methods.add_async_method_mut("write_backend", |_, this, data: Vec<u8>| async move {
            Self::to_lua_result(Self::write_stream(&mut this.backend_stream, &data).await)
        });

        methods.add_async_method_mut("read_client_str", |_, this, size: usize| async move {
            let buf = Self::to_lua_result(
                Self::read_stream(&mut this.client_stream, size, this.buf_size).await,
            )?;
            Ok(String::from_utf8(buf).unwrap())
        });

        methods.add_async_method_mut("write_client_str", |_, this, data: String| async move {
            Self::to_lua_result(Self::write_stream(&mut this.client_stream, data.as_bytes()).await)
        });

        methods.add_async_method_mut("read_backend_str", |_, this, size: usize| async move {
            let buf = Self::to_lua_result(
                Self::read_stream(&mut this.backend_stream, size, this.buf_size).await,
            )?;
            Ok(String::from_utf8(buf).unwrap())
        });

        methods.add_async_method_mut("write_backend_str", |_, this, data: String| async move {
            Self::to_lua_result(Self::write_stream(&mut this.backend_stream, data.as_bytes()).await)
        });

        methods.add_async_method_mut("close_client", |_, this, ()| async move {
            Self::to_lua_result(Self::close_stream(&mut this.client_stream).await)
        });

        methods.add_async_method_mut("close_backend", |_, this, ()| async move {
            Self::to_lua_result(Self::close_stream(&mut this.backend_stream).await)
        });

        methods.add_async_method_mut("close_all", |_, this, ()| async move {
            Self::to_lua_result(Self::close_stream(&mut this.client_stream).await)?;
            Self::to_lua_result(Self::close_stream(&mut this.backend_stream).await)
        });

        methods.add_async_method_mut("close", |_, this, ()| async move {
            Self::to_lua_result(Self::close_stream(&mut this.client_stream).await)?;
            Self::to_lua_result(Self::close_stream(&mut this.backend_stream).await)
        });

        // read request or backend with select
        methods.add_async_method_mut("select", |_, this, size: usize| async move {
            let size = if size > 0 { size } else { this.buf_size };
            let mut buf = vec![0; size];

            tokio::select! {
                _ = this.client_stream.readable() => {
                    match this.client_stream.read(&mut buf).await {
                        Ok(n) if n > 0 => Ok(("client", buf[..n].to_vec())),
                        _ => {
                            Self::to_lua_result(Self::close_stream(&mut this.client_stream).await)?;
                            this.backend_stream.shutdown().await?;
                            Ok(("client", vec![]))
                        }
                    }
                },
                _ = this.backend_stream.readable() => {
                    match this.backend_stream.read(&mut buf).await {
                        Ok(n) if n > 0 => Ok(("backend", buf[..n].to_vec())),
                        _ => {
                            this.client_stream.shutdown().await?;
                            this.backend_stream.shutdown().await?;
                            Ok(("backend", vec![]))
                        }
                    }
                }
            }
        });
    }
}
