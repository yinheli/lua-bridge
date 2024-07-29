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
luarocks install lua-cjson
luarocks install serpent
```

### 安装到非全局

```bash
luarocks install lua-cjson --tree $PWD/.rocks
luarocks install serpent --tree $PWD/.rocks
```

安装到当前目录的 .rocks 目录下, 查看目录结构

```bash
tree $PWD/.rocks
```

```
|-- bin
|   |-- json2lua
|   `-- lua2json
|-- lib
|   |-- lua
|   |   `-- 5.1
|   |       `-- cjson.so
|   `-- luarocks
|       `-- rocks-5.1
|           |-- lua-cjson
|           |   `-- 2.1.0.10-1
|           |       |-- bin
|           |       |   |-- json2lua
|           |       |   `-- lua2json
|           |       |-- lua-cjson-2.1.0.10-1.rockspec
|           |       |-- rock_manifest
|           |       `-- tests
|           |           |-- README
|           |           |-- TestLua.pm
|           |           |-- agentzh.t
|           |           |-- bench.lua
|           |           |-- example1.json
|           |           |-- example2.json
|           |           |-- example3.json
|           |           |-- example4.json
|           |           |-- example5.json
|           |           |-- genutf8.pl
|           |           |-- numbers.json
|           |           |-- octets-escaped.dat
|           |           |-- rfc-example1.json
|           |           |-- rfc-example2.json
|           |           |-- sort_json.lua
|           |           |-- test.lua
|           |           `-- types.json
|           `-- manifest
`-- share
    `-- lua
        `-- 5.1
            |-- cjson
            |   `-- util.lua
            |-- json2lua.lua
            `-- lua2json.lua

15 directories, 28 files
```


```bash
export LUA_PATH="$PWD/.rocks/share/lua/5.1/?.lua;$PWD/my_project_rocks/share/lua/5.1/?/init.lua;;"
export LUA_CPATH="$PWD/.rocks/lib/lua/5.1/?.so;;"
```

> 可以把上述环境变量写入到 env.sh 中，然后 source env.sh
