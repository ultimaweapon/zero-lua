#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <new>
#include <type_traits>
#include <utility>

#include <stdint.h>
#include <string.h>

static_assert(sizeof(lua_Integer) == sizeof(int64_t));
static_assert(std::is_signed<lua_Integer>::value);
static_assert(LUA_EXTRASPACE == sizeof(void *));

extern "C" int ZL_REGISTRYINDEX = LUA_REGISTRYINDEX;

extern "C" lua_State *lua54_newstate()
{
    // Create Lua state.
    auto L = luaL_newstate();

    if (!L) {
        throw std::bad_alloc();
    }

    // Lua does not mention about the initial content of extra space and it seems like Lua does not
    // zeroed this area.
    memset(lua_getextraspace(L), 0, LUA_EXTRASPACE);

    // Register libraries that does not need to alter its behavior.
    auto libs = {
        std::make_pair(LUA_TABLIBNAME, luaopen_table),
        std::make_pair(LUA_STRLIBNAME, luaopen_string),
        std::make_pair(LUA_MATHLIBNAME, luaopen_math),
        std::make_pair(LUA_UTF8LIBNAME, luaopen_utf8)
    };

    for (auto &l : libs) {
        luaL_requiref(L, l.first, l.second, 1);
        lua_pop(L, 1);
    }

    return L;
}

extern "C" void zl_close(lua_State *L)
{
    lua_close(L);
}

extern "C" void zl_require_base(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_GNAME, luaopen_base, global);
}

extern "C" void zl_require_coroutine(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_COLIBNAME, luaopen_coroutine, global);
}

extern "C" void zl_require_io(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_IOLIBNAME, luaopen_io, global);
}

extern "C" void zl_require_os(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_OSLIBNAME, luaopen_os, global);
}

extern "C" bool zl_load(lua_State *L, const char *name, const char *chunk, size_t len)
{
    return luaL_loadbuffer(L, chunk, len, name) == LUA_OK;
}

extern "C" bool zl_pcall(lua_State *L, int nargs, int nresults, int msgh)
{
    return lua_pcall(L, nargs, nresults, msgh) == LUA_OK;
}

extern "C" void engine_checkstack(lua_State *L, int n)
{
    luaL_checkstack(L, n, nullptr);
}

extern "C" void engine_pushnil(lua_State *L)
{
    lua_pushnil(L);
}

extern "C" const char *zl_pushlstring(lua_State *L, const char *s, size_t len)
{
    return lua_pushlstring(L, s, len);
}

extern "C" void engine_pushcclosure(lua_State *L, int (*fn) (lua_State *L), int n)
{
    lua_pushcclosure(L, fn, n);
}

extern "C" int engine_gettop(lua_State *L)
{
    return lua_gettop(L);
}

extern "C" const char *zl_checklstring(lua_State *L, int arg, size_t *l)
{
    return luaL_checklstring(L, arg, l);
}

extern "C" void lua54_typeerror(lua_State *L, int arg, const char *tname)
{
    luaL_typeerror(L, arg, tname);
}

extern "C" void engine_argerror(lua_State *L, int arg, const char *extramsg)
{
    luaL_argerror(L, arg, extramsg);
}

extern "C" bool engine_isnil(lua_State *L, int index)
{
    return lua_isnil(L, index) != 0;
}

extern "C" bool lua54_istable(lua_State *L, int index)
{
    return lua_istable(L, index) != 0;
}

extern "C" int64_t engine_tointegerx(lua_State *L, int index, int *isnum)
{
    return static_cast<int64_t>(lua_tointegerx(L, index, isnum));
}

extern "C" const char *zl_tolstring(lua_State *L, int index, size_t *len)
{
    return lua_tolstring(L, index, len);
}

extern "C" void *engine_touserdata(lua_State *L, int index)
{
    return lua_touserdata(L, index);
}

extern "C" int lua54_type(lua_State *L, int index)
{
    return lua_type(L, index);
}

extern "C" const char *lua54_typename(lua_State *L, int tp)
{
    return lua_typename(L, tp);
}

extern "C" void engine_createtable(lua_State *L, int narr, int nrec)
{
    lua_createtable(L, narr, nrec);
}

extern "C" int zl_ref(lua_State *L, int t)
{
    return luaL_ref(L, t);
}

extern "C" void zl_unref(lua_State *L, int t, int ref)
{
    luaL_unref(L, t, ref);
}

extern "C" int lua54_geti(lua_State *L, int index, int64_t i)
{
    return lua_geti(L, index, i);
}

extern "C" void lua54_seti(lua_State *L, int index, int64_t n)
{
    lua_seti(L, index, n);
}

extern "C" int lua54_getfield(lua_State *L, int index, const char *k)
{
    return lua_getfield(L, index, k);
}

extern "C" void engine_setfield(lua_State *L, int index, const char *k)
{
    lua_setfield(L, index, k);
}

extern "C" void *engine_newuserdatauv(lua_State *L, size_t size, int nuvalue)
{
    return lua_newuserdatauv(L, size, nuvalue);
}

extern "C" int zl_newmetatable(lua_State *L, const char *tname)
{
    return luaL_newmetatable(L, tname);
}

extern "C" int zl_globalmetatable(lua_State *L, const char *tname)
{
    return luaL_getmetatable(L, tname);
}

extern "C" void zl_setmetatable(lua_State *L, int index)
{
    lua_setmetatable(L, index);
}

extern "C" int zl_getmetatable(lua_State *L, int index)
{
    return lua_getmetatable(L, index);
}

extern "C" int zl_getmetafield(lua_State *L, int obj, const char *e)
{
    return luaL_getmetafield(L, obj, e);
}

extern "C" int engine_upvalueindex(int i)
{
    return lua_upvalueindex(i);
}

extern "C" void lua54_setglobal(lua_State *L, const char *name)
{
    lua_setglobal(L, name);
}

extern "C" void lua54_replace(lua_State *L, int index)
{
    lua_replace(L, index);
}

extern "C" void engine_pop(lua_State *L, int n)
{
    lua_pop(L, n);
}

extern "C" int zl_error(lua_State *L, const char *msg)
{
    return luaL_error(L, "%s", msg);
}

extern "C" void *zl_getextraspace(lua_State *L)
{
    return lua_getextraspace(L);
}

extern "C" lua_State *zl_newthread(lua_State *L)
{
    return lua_newthread(L);
}
