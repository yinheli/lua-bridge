# lua bridge

## 需求

- Rust tcp 代理，数据流转到 lua 脚本

## 配置

```env
# 本地监听地址
LISTEN=0.0.0.0:5000

# 后端服务地址
BACKEND=127.0.0.1:8081

# MySQL 数据库
MYSQL_URI=

# Redis 数据库
REDIS_URI=

# Lua 脚本文件路径
SCRIPT=./app.lua

# Lua 脚本入口函数名
SCRIPT_ENTRY=handle
```


## 函数

lua 脚本

全局函数， 参考 script 中的 register_xxx 函数注入到 lua 中的全局函数或者常量。

入口函数, 传入参数为 `ctx`，类型为 `table`，包含的方法参考 script 中 Ctx 结构，UserData 实现的的 add_methods 方法。

- read_client
- read_backend
- write_client
- write_backend
- close_client
- close_backend
- ......

```lua
function handle(ctx)

end
```

## 依赖管理

以 Debian 12 为为例

```bash
apt-get install -y build-essential git libssl-dev lua5.1 liblua5.1-dev

apt-get install -y luarocks
```

```bash
# 安装依赖，比如
# 更多信息参考 https://luarocks.org/
luarocks install cjson
luarocks install serpent
```
