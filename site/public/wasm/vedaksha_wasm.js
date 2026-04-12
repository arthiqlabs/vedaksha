/* @ts-self-types="./vedaksha_wasm.d.ts" */

/**
 * Compute Vimshottari Dasha periods from Moon's sidereal longitude.
 *
 * # Arguments
 * * `moon_longitude` — Moon's sidereal longitude in degrees [0, 360)
 * * `birth_jd` — Julian Day of birth
 * * `levels` — Depth of sub-periods (1-5, default 3)
 *
 * # Returns
 * JSON string with the complete dasha tree.
 * @param {number} moon_longitude
 * @param {number} birth_jd
 * @param {number} levels
 * @returns {string}
 */
export function compute_dasha(moon_longitude, birth_jd, levels) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.compute_dasha(moon_longitude, birth_jd, levels);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Compute house cusps.
 *
 * # Arguments
 * * `ramc` — Right Ascension of MC in degrees
 * * `latitude` — Geographic latitude in degrees
 * * `obliquity` — Obliquity of the ecliptic in degrees
 * * `system` — House system: "Placidus", "Equal", "WholeSign", etc.
 *
 * # Returns
 * JSON string with 12 cusp longitudes, ASC, MC.
 * @param {number} ramc
 * @param {number} latitude
 * @param {number} obliquity
 * @param {string} system
 * @returns {string}
 */
export function compute_houses(ramc, latitude, obliquity, system) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(system, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compute_houses(ramc, latitude, obliquity, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Compute a complete natal chart from birth data.
 *
 * # Arguments
 * * `config_json` — JSON string with birth data and optional configuration.
 *
 * Required: `year`, `month`, `day`, `hour`, `minute`, `latitude`, `longitude`
 * Optional: `second` (0), `ayanamsha` ("Lahiri"), `house_system` ("Placidus"),
 *           `bodies` (default 9 Jyotish graha + nodes)
 *
 * Input datetime is UTC.
 *
 * # Returns
 * JSON string with planets, houses, aspects, ayanamsha value, Julian Day.
 * @param {string} config_json
 * @returns {string}
 */
export function compute_natal_chart(config_json) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(config_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.compute_natal_chart(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Compute the varga (divisional chart) sign for a longitude.
 *
 * # Arguments
 * * `longitude` — Sidereal longitude in degrees
 * * `varga` — Varga name: "Rashi", "Navamsha", "Dashamsha", etc.
 *
 * # Returns
 * Sign index (0-11) in the divisional chart.
 * @param {number} longitude
 * @param {string} varga
 * @returns {number}
 */
export function compute_varga(longitude, varga) {
    const ptr0 = passStringToWasm0(varga, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.compute_varga(longitude, ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * Find aspects between a set of planetary positions.
 *
 * # Arguments
 * * `positions_json` — JSON array of {longitude: number, speed: number}
 * * `major_only` — If true, only check major (Ptolemaic) aspects
 *
 * # Returns
 * JSON string with array of detected aspects.
 * @param {string} positions_json
 * @param {boolean} major_only
 * @returns {string}
 */
export function find_aspects(positions_json, major_only) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(positions_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.find_aspects(ptr0, len0, major_only);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Get the ayanamsha value in degrees for a given date.
 * @param {string} ayanamsha
 * @param {number} jd
 * @returns {number}
 */
export function get_ayanamsha(ayanamsha, jd) {
    const ptr0 = passStringToWasm0(ayanamsha, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.get_ayanamsha(ptr0, len0, jd);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * Get the nakshatra and pada for a sidereal longitude.
 *
 * # Arguments
 * * `sidereal_longitude` — Sidereal longitude in degrees [0, 360)
 *
 * # Returns
 * JSON string with nakshatra name, index, pada, dasha lord.
 * @param {number} sidereal_longitude
 * @returns {string}
 */
export function get_nakshatra(sidereal_longitude) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.get_nakshatra(sidereal_longitude);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Get the zodiac sign for a longitude.
 *
 * # Returns
 * JSON with sign name and index.
 * @param {number} longitude
 * @returns {string}
 */
export function get_sign(longitude) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.get_sign(longitude);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * Get localized name for a nakshatra.
 * @param {number} index
 * @param {string} language
 * @returns {string}
 */
export function nakshatra_name(index, language) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nakshatra_name(index, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Get localized name for a planet.
 * @param {number} index
 * @param {string} language
 * @returns {string}
 */
export function planet_name(index, language) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.planet_name(index, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Get localized name for a zodiac sign.
 * @param {number} index
 * @param {string} language
 * @returns {string}
 */
export function sign_name(index, language) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(language, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.sign_name(index, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Convert tropical longitude to sidereal.
 *
 * # Arguments
 * * `tropical_longitude` — Tropical longitude in degrees
 * * `ayanamsha` — Ayanamsha system: "Lahiri", "FaganBradley", "Krishnamurti", etc.
 * * `jd` — Julian Day for computation
 * @param {number} tropical_longitude
 * @param {string} ayanamsha
 * @param {number} jd
 * @returns {number}
 */
export function tropical_to_sidereal(tropical_longitude, ayanamsha, jd) {
    const ptr0 = passStringToWasm0(ayanamsha, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.tropical_to_sidereal(tropical_longitude, ptr0, len0, jd);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_960c155d3d49e4c2: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./vedaksha_wasm_bg.js": import0,
    };
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('vedaksha_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
