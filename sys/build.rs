use flate2::read::MultiGzDecoder;
use std::path::{MAIN_SEPARATOR_STR, Path};

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Check if we have Lua source.
    let path = Path::new("lua-5.4.7");

    if !path.exists() {
        // Download and extract source.
        let tar = ureq::get("https://www.lua.org/ftp/lua-5.4.7.tar.gz")
            .call()
            .unwrap()
            .into_body()
            .into_reader();
        let tar = MultiGzDecoder::new(tar);
        let mut tar = tar::Archive::new(tar);

        tar.unpack(&root).unwrap();

        // Get path to lapi.h.
        let mut path = path.join("src");

        path.push("lapi.h");

        // Patch api_incr_top.
        let content = std::fs::read_to_string(&path).unwrap();
        let content = content.replacen(
            "/* Increments 'L->top.p', checking for stack overflows */\n#define api_incr_top(L)	{L->top.p++; \\\n			 api_check(L, L->top.p <= L->ci->top.p, \\\n					\"stack overflow\");}",
            r#"extern "C" [[noreturn]] void zl_panic(const char *msg) noexcept;

/* Increments 'L->top.p', checking for stack overflows */
#define api_incr_top(L)	{L->top.p++; if (L->top.p > L->ci->top.p) zl_panic("Lua stack overflow");}"#,
            1,
        );

        std::fs::write(path, content).unwrap();
    }

    // Setup builder.
    let lua = format!("lua-5.4.7{MAIN_SEPARATOR_STR}src");
    let mut cc = cc::Build::new();
    let sources = [
        "lapi.c",
        "lcode.c",
        "lctype.c",
        "ldebug.c",
        "ldo.c",
        "ldump.c",
        "lfunc.c",
        "lgc.c",
        "llex.c",
        "lmem.c",
        "lobject.c",
        "lopcodes.c",
        "lparser.c",
        "lstate.c",
        "lstring.c",
        "ltable.c",
        "ltm.c",
        "lundump.c",
        "lvm.c",
        "lzio.c",
        "lauxlib.c",
        "lbaselib.c",
        "lcorolib.c",
        "ldblib.c",
        "liolib.c",
        "lmathlib.c",
        "loadlib.c",
        "loslib.c",
        "lstrlib.c",
        "ltablib.c",
        "lutf8lib.c",
        "linit.c",
    ];

    // Use C++ exception instead of setjmp/longjmp for error/yield.
    cc.cpp(true);

    if cc.get_compiler().is_like_msvc() {
        cc.flag("/TP");
    } else {
        cc.flag("-xc++");
    }

    match os.as_str() {
        "linux" => cc.define("LUA_USE_LINUX", None),
        "macos" => cc.define("LUA_USE_MACOSX", None),
        "windows" => &mut cc,
        _ => panic!("target OS is not supported"),
    };

    for src in sources {
        cc.file(format!("{lua}{MAIN_SEPARATOR_STR}{src}"));
    }

    cc.compile("lua");

    // Export include path.
    println!("cargo::metadata=include_path={root}{MAIN_SEPARATOR_STR}{lua}");
}
