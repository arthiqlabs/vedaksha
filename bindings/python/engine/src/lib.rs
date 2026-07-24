//! C-ABI wasm host shim for the Vedākṣha engine.
//!
//! This compiles the real `vedaksha-ephem-core` and `vedaksha-mcp` crates to a
//! `wasm32-unknown-unknown` module with a small, stable C ABI. The Python
//! package loads the resulting `vedaksha.wasm` via `wasmtime` and drives it
//! through these exports. No engine logic lives here or in Python — this is a
//! transport, and the numbers it returns are the Rust engine's own, bit-for-bit
//! (proven by the SPK-over-wasm spike, docs/adr/2026-07-23-spk-over-wasm-spike.md).
//!
//! ## Memory contract
//!
//! The module imports nothing (no WASI). The host manages buffers inside our
//! linear memory: call [`vk_alloc`] to reserve `n` bytes, write into
//! `memory[ptr..ptr+n]`, pass `(ptr, n)` to a function, and [`vk_free`] when
//! done. Every function returns an `i32`: `>= 0` is success (often a length),
//! negative is an error code.
//!
//! ## Two surfaces
//!
//! * **MCP** — [`vk_mcp_request`]/[`vk_mcp_take`] wrap `McpServer::handle_request`,
//!   exposing all 15 tools as JSON-RPC 2.0. The Python library, CLI and REST
//!   layers are all projections of this one entry point (analytical ephemeris
//!   tier, no external data).
//! * **SPK** — [`vk_spk_load`]/[`vk_spk_state`] expose the sub-arcsecond
//!   `SpkReader` for callers that supply a DE440s kernel. Independent of MCP.

use std::sync::Mutex;

use vedaksha_ephem_core::bodies::Body;
use vedaksha_ephem_core::jpl::{EphemerisProvider, reader::SpkReader};

/// ABI version of this shim. Bump on any breaking change to the exports below.
const ABI_VERSION: i32 = 1;

// Error codes (negative). Kept in sync with `_abi.py`.
const ERR_UTF8: i32 = -1;
const ERR_UNKNOWN_BODY: i32 = -2;
const ERR_NO_READER: i32 = -3;
const ERR_COMPUTE: i32 = -4;
const ERR_PARSE: i32 = -5;
const ERR_NO_PENDING: i32 = -6;
const ERR_CAPACITY: i32 = -7;

static SPK: Mutex<Option<SpkReader>> = Mutex::new(None);
static PENDING: Mutex<Option<Vec<u8>>> = Mutex::new(None);

/// Return the ABI version of this module.
#[unsafe(no_mangle)]
pub extern "C" fn vk_abi_version() -> i32 {
    ABI_VERSION
}

/// Reserve `len` bytes in linear memory; returns a pointer the host may write to.
#[unsafe(no_mangle)]
pub extern "C" fn vk_alloc(len: usize) -> *mut u8 {
    let mut buf = Vec::<u8>::with_capacity(len);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

/// Release a buffer previously returned by [`vk_alloc`].
///
/// # Safety
/// `ptr`/`len` must come from a prior [`vk_alloc`] call and not be reused after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
    }
}

// --- MCP surface -----------------------------------------------------------

/// Handle one JSON-RPC 2.0 request at `req[ptr..ptr+len]`. On success the
/// response is buffered internally and its byte length returned; the host then
/// calls [`vk_mcp_take`] to copy it out. Returns [`ERR_UTF8`] on invalid input.
///
/// # Safety
/// `ptr`/`len` must describe an initialised, readable region of linear memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_mcp_request(ptr: *const u8, len: usize) -> i32 {
    let raw = unsafe { core::slice::from_raw_parts(ptr, len) };
    let Ok(req) = core::str::from_utf8(raw) else {
        return ERR_UTF8;
    };
    let server = vedaksha_mcp::server::McpServer::new();
    let resp = server.handle_request(req).into_bytes();
    let n = resp.len() as i32;
    *PENDING.lock().unwrap() = Some(resp);
    n
}

/// Copy the buffered response from the last [`vk_mcp_request`] into
/// `out[..cap]`. Returns the number of bytes written, [`ERR_NO_PENDING`] if no
/// response is buffered, or [`ERR_CAPACITY`] if `cap` is too small.
///
/// # Safety
/// `out`/`cap` must describe a writable region of linear memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_mcp_take(out: *mut u8, cap: usize) -> i32 {
    let mut guard = PENDING.lock().unwrap();
    let Some(buf) = guard.take() else {
        return ERR_NO_PENDING;
    };
    if buf.len() > cap {
        *guard = Some(buf);
        return ERR_CAPACITY;
    }
    let dst = unsafe { core::slice::from_raw_parts_mut(out, buf.len()) };
    dst.copy_from_slice(&buf);
    buf.len() as i32
}

// --- SPK surface -----------------------------------------------------------

/// Parse a DE440s (SPK/DAF) image at `raw[ptr..ptr+len]` and install it as the
/// active ephemeris. Returns 0 on success, [`ERR_PARSE`] on a malformed file.
///
/// # Safety
/// `ptr`/`len` must describe an initialised, readable region of linear memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_spk_load(ptr: *const u8, len: usize) -> i32 {
    let raw = unsafe { core::slice::from_raw_parts(ptr, len) };
    match SpkReader::from_bytes(raw) {
        Ok(r) => {
            *SPK.lock().unwrap() = Some(r);
            0
        }
        Err(_) => ERR_PARSE,
    }
}

/// True (1) if an SPK kernel is currently loaded, else 0.
#[unsafe(no_mangle)]
pub extern "C" fn vk_spk_loaded() -> i32 {
    i32::from(SPK.lock().unwrap().is_some())
}

fn body_from_naif(id: i32) -> Option<Body> {
    Some(match id {
        1 => Body::Mercury,
        2 => Body::Venus,
        3 => Body::EarthMoonBarycenter,
        4 => Body::Mars,
        5 => Body::Jupiter,
        6 => Body::Saturn,
        7 => Body::Uranus,
        8 => Body::Neptune,
        9 => Body::Pluto,
        10 => Body::Sun,
        301 => Body::Moon,
        _ => return None,
    })
}

/// Compute the state vector of the NAIF body `naif_id` at Julian Day `jd`,
/// writing `[x, y, z, vx, vy, vz]` (AU, AU/day, ICRS) to `out[..6]`.
///
/// Returns 0 on success, or a negative error: [`ERR_UNKNOWN_BODY`],
/// [`ERR_NO_READER`] (no kernel loaded), [`ERR_COMPUTE`] (out of range, etc.).
///
/// # Safety
/// `out` must point to space for 6 `f64` values in linear memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_spk_state(naif_id: i32, jd: f64, out: *mut f64) -> i32 {
    let Some(body) = body_from_naif(naif_id) else {
        return ERR_UNKNOWN_BODY;
    };
    let guard = SPK.lock().unwrap();
    let Some(reader) = guard.as_ref() else {
        return ERR_NO_READER;
    };
    match reader.compute_state(body, jd) {
        Ok(s) => {
            let dst = unsafe { core::slice::from_raw_parts_mut(out, 6) };
            dst[0] = s.position.x;
            dst[1] = s.position.y;
            dst[2] = s.position.z;
            dst[3] = s.velocity.x;
            dst[4] = s.velocity.y;
            dst[5] = s.velocity.z;
            0
        }
        Err(_) => ERR_COMPUTE,
    }
}

/// Write the loaded kernel's JD coverage `[jd_min, jd_max]` to `out[..2]`.
/// Returns 0, or [`ERR_NO_READER`] if no kernel is loaded.
///
/// # Safety
/// `out` must point to space for 2 `f64` values in linear memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vk_spk_range(out: *mut f64) -> i32 {
    let guard = SPK.lock().unwrap();
    let Some(reader) = guard.as_ref() else {
        return ERR_NO_READER;
    };
    let (lo, hi) = reader.time_range();
    let dst = unsafe { core::slice::from_raw_parts_mut(out, 2) };
    dst[0] = lo;
    dst[1] = hi;
    0
}
