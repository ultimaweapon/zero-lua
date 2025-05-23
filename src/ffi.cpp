#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

#include <type_traits>

#include <stdint.h>
#include <string.h>

static_assert(sizeof(lua_Integer) == sizeof(int64_t));
static_assert(std::is_signed<lua_Integer>::value);
static_assert(sizeof(lua_KContext) == sizeof(intptr_t));
static_assert(LUA_EXTRASPACE == sizeof(void *) * 2);
static_assert(LUA_MINSTACK == 20);
static_assert(LUA_MULTRET == -1);

extern "C" {
    int ZL_REGISTRYINDEX = LUA_REGISTRYINDEX;
    const char *ZL_LOADED_TABLE = LUA_LOADED_TABLE;
}

extern "C" lua_State *zl_newstate()
{
    auto L = luaL_newstate();

    if (L) {
        memset(lua_getextraspace(L), 0, LUA_EXTRASPACE);
    }

    return L;
}

extern "C" void zl_close(lua_State *L)
{
    lua_close(L);
}

extern "C" lua_CFunction zl_atpanic(lua_State *L, int (*panicf) (lua_State *L))
{
    return lua_atpanic(L, panicf);
}

extern "C" void zl_require_base(lua_State *L)
{
    luaL_requiref(L, LUA_GNAME, luaopen_base, 0);
}

extern "C" void zl_require_coroutine(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_COLIBNAME, luaopen_coroutine, global);
}

extern "C" void zl_require_io(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_IOLIBNAME, luaopen_io, global);
}

extern "C" void zl_require_math(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_MATHLIBNAME, luaopen_math, global);
}

extern "C" void zl_require_os(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_OSLIBNAME, luaopen_os, global);
}

extern "C" void zl_require_string(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_STRLIBNAME, luaopen_string, global);
}

extern "C" void zl_require_table(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_TABLIBNAME, luaopen_table, global);
}

extern "C" void zl_require_utf8(lua_State *L, bool global)
{
    luaL_requiref(L, LUA_UTF8LIBNAME, luaopen_utf8, global);
}

extern "C" bool zl_load(lua_State *L, const char *name, const char *chunk, size_t len, const char *mode)
{
    return luaL_loadbufferx(L, chunk, len, name, mode) == LUA_OK;
}

extern "C" bool zl_pcall(lua_State *L, int nargs, int nresults, int msgh)
{
    return lua_pcall(L, nargs, nresults, msgh) == LUA_OK;
}

extern "C" void zl_checkstack(lua_State *L, int n)
{
    luaL_checkstack(L, n, nullptr);
}

extern "C" void zl_pushnil(lua_State *L)
{
    lua_pushnil(L);
}

extern "C" void zl_pushboolean(lua_State *L, bool b)
{
    lua_pushboolean(L, b);
}

extern "C" const char *zl_pushlstring(lua_State *L, const char *s, size_t len)
{
    return lua_pushlstring(L, s, len);
}

extern "C" void zl_pushlightuserdata(lua_State *L, void *p)
{
    lua_pushlightuserdata(L, p);
}

extern "C" void zl_pushcclosure(lua_State *L, int (*fn) (lua_State *L), int n)
{
    lua_pushcclosure(L, fn, n);
}

extern "C" void zl_pushvalue(lua_State *L, int index)
{
    return lua_pushvalue(L, index);
}

extern "C" int zl_gettop(lua_State *L)
{
    return lua_gettop(L);
}

extern "C" const char *zl_checklstring(lua_State *L, int arg, size_t *l)
{
    return luaL_checklstring(L, arg, l);
}

extern "C" void zl_typeerror(lua_State *L, int arg, const char *tname)
{
    luaL_typeerror(L, arg, tname);
}

extern "C" void zl_argerror(lua_State *L, int arg, const char *extramsg)
{
    luaL_argerror(L, arg, extramsg);
}

extern "C" bool zl_isnil(lua_State *L, int index)
{
    return lua_isnil(L, index) != 0;
}

extern "C" bool zl_istable(lua_State *L, int index)
{
    return lua_istable(L, index) != 0;
}

extern "C" int64_t zl_tointegerx(lua_State *L, int index, int *isnum)
{
    return static_cast<int64_t>(lua_tointegerx(L, index, isnum));
}

extern "C" const char *zl_tolstring(lua_State *L, int index, size_t *len)
{
    return lua_tolstring(L, index, len);
}

extern "C" void *zl_touserdata(lua_State *L, int index)
{
    return lua_touserdata(L, index);
}

extern "C" int zl_type(lua_State *L, int index)
{
    return lua_type(L, index);
}

extern "C" const char *zl_typename(lua_State *L, int tp)
{
    return lua_typename(L, tp);
}

extern "C" void zl_createtable(lua_State *L, int narr, int nrec)
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

extern "C" int zl_geti(lua_State *L, int index, int64_t i)
{
    return lua_geti(L, index, i);
}

extern "C" void zl_seti(lua_State *L, int index, int64_t n)
{
    lua_seti(L, index, n);
}

extern "C" int zl_getfield(lua_State *L, int index, const char *k)
{
    return lua_getfield(L, index, k);
}

extern "C" void zl_setfield(lua_State *L, int index, const char *k)
{
    lua_setfield(L, index, k);
}

extern "C" int zl_getsubtable(lua_State *L, int idx, const char *fname)
{
    return luaL_getsubtable(L, idx, fname);
}

extern "C" void *zl_newuserdatauv(lua_State *L, size_t size, int nuvalue)
{
    return lua_newuserdatauv(L, size, nuvalue);
}

extern "C" int zl_setiuservalue(lua_State *L, int index, uint16_t n)
{
    return lua_setiuservalue(L, index, n);
}

extern "C" int zl_getiuservalue(lua_State *L, int index, uint16_t n)
{
    return lua_getiuservalue(L, index, n);
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

extern "C" int zl_upvalueindex(int i)
{
    return lua_upvalueindex(i);
}

extern "C" void zl_setglobal(lua_State *L, const char *name)
{
    lua_setglobal(L, name);
}

extern "C" void zl_replace(lua_State *L, int index)
{
    lua_replace(L, index);
}

extern "C" void zl_pop(lua_State *L, int n)
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

extern "C" int zl_resume(lua_State *L, lua_State *from, int nargs, int *nresults)
{
    return lua_resume(L, from, nargs, nresults);
}

extern "C" int zl_yieldk(lua_State *L, int nresults, intptr_t ctx, int (*k) (lua_State *L, int status, intptr_t ctx))
{
    return lua_yieldk(L, nresults, ctx, k);
}
