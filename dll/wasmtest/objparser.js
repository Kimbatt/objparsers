
const objParser = (() =>
{
    let wasm = null;

    const waitingPromisesResolveFunctions = [];
    async function ensureLoaded()
    {
        wasm === null && await new Promise(resolve => waitingPromisesResolveFunctions.push(resolve));
    }

    (async () =>
    {
        const response = await fetch("../pkg/dll_bg.wasm");
        const buffer = await response.arrayBuffer();
        wasm = (await WebAssembly.instantiate(buffer)).instance.exports;
        console.log(wasm);

        waitingPromisesResolveFunctions.forEach(resolve => resolve());
    })();

    let cachegetUint8Memory0 = null;
    function getUint8Memory0()
    {
        if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== wasm.memory.buffer)
        {
            cachegetUint8Memory0 = new Uint8Array(wasm.memory.buffer);
        }

        return cachegetUint8Memory0;
    }

    let WASM_VECTOR_LEN = 0;

    function passArray8ToWasm0(arg, malloc)
    {
        const ptr = malloc(arg.length * 1);
        getUint8Memory0().set(arg, ptr / 1);
        WASM_VECTOR_LEN = arg.length;
        return ptr;
    }

    /**
     * @param {Uint8Array} file_content_bytes
     * @returns {number}
     */
    function wasm_parse_obj(file_content_bytes)
    {
        var ptr0 = passArray8ToWasm0(file_content_bytes, wasm.__wbindgen_malloc);
        var len0 = WASM_VECTOR_LEN;
        var ret = wasm.wasm_parse_obj(ptr0, len0);
        return ret;
    }

    /**
     * @param {number} handle
     * @returns {number}
     */
    function wasm_get_vertex_count(handle)
    {
        var ret = wasm.wasm_get_vertex_count(handle);
        return ret >>> 0;
    }

    /**
     * @param {number} handle
     * @returns {number}
     */
    function wasm_get_vertex_positions(handle)
    {
        var ret = wasm.wasm_get_vertex_positions(handle);
        return ret;
    }

    /**
     * @param {number} handle
     * @returns {number}
     */
    function wasm_get_index_count(handle)
    {
        var ret = wasm.wasm_get_index_count(handle);
        return ret >>> 0;
    }

    /**
     * @param {number} handle
     * @returns {number}
     */
    function wasm_get_indices(handle)
    {
        var ret = wasm.wasm_get_indices(handle);
        return ret;
    }

    /**
     * @param {number} handle
     */
    function wasm_destroy_handle(handle)
    {
        wasm.wasm_destroy_handle(handle);
    }

    return {
        wasm_parse_obj: wasm_parse_obj,
        wasm_get_vertex_count: wasm_get_vertex_count,
        wasm_get_vertex_positions: wasm_get_vertex_positions,
        wasm_get_index_count: wasm_get_index_count,
        wasm_get_indices: wasm_get_indices,
        wasm_destroy_handle: wasm_destroy_handle,
        ensureLoaded: ensureLoaded,

        /**
         * @returns {ArrayBuffer}
         */
        memory: () => wasm.memory.buffer
    };
})();

/**
 * @param {Uint8Array} bytes
 */
async function ParseObj(bytes)
{
    await objParser.ensureLoaded();

    const start = performance.now();
    const handle = objParser.wasm_parse_obj(bytes);
    const end = performance.now();
    console.log(`obj parsed in ${end - start}ms`);

    if (handle === 0)
    {
        return null;
    }

    try
    {
        const memory = objParser.memory();
        const f32memory = new Float32Array(memory);
        const u32memory = new Uint32Array(memory);

        const vertexCount = objParser.wasm_get_vertex_count(handle);
        const verticesPtr = objParser.wasm_get_vertex_positions(handle);
        const vertexStartIndex = verticesPtr >>> 2;
        const vertices = f32memory.slice(vertexStartIndex, vertexStartIndex + vertexCount * 3);

        const indexCount = objParser.wasm_get_index_count(handle);
        const indexPtr = objParser.wasm_get_indices(handle);
        const indexStartIndex = indexPtr >>> 2;
        const indices = u32memory.slice(indexStartIndex, indexStartIndex + indexCount);

        return {
            vertices: vertices,
            indices: indices
        };
    }
    finally
    {
        objParser.wasm_destroy_handle(handle);
    }
}

/**
 * @param {Float32Array} vertices
 * @param {Uint32Array} indices
 */
function WriteObj(vertices, indices)
{
    /**
     * @type {string[]}
     */
    const rows = [];

    for (let i = 0; i < vertices.length; i += 3)
    {
        const v0 = vertices[i];
        const v1 = vertices[i + 1];
        const v2 = vertices[i + 2];
        rows.push(`v ${v0.toString()} ${v1.toString()} ${v2.toString()}`);
    }

    for (let i = 0; i < indices.length; i += 3)
    {
        const f0 = indices[i] + 1;
        const f1 = indices[i + 1] + 1;
        const f2 = indices[i + 2] + 1;
        rows.push(`f ${f0.toString()} ${f1.toString()} ${f2.toString()}`);
    }

    const blob = new Blob([new Uint8Array(rows.join("\n").split("").map(c => c.charCodeAt(0)))]);
    const blobUrl = window.URL.createObjectURL(blob);

    const a = document.createElement("a");
    a.download = "exported.obj";
    a.href = blobUrl;

    a.click();
}

/**
 * @param {HTMLInputElement} inputElement
 */
function FileSelected(inputElement)
{
    const files = inputElement.files;
    const fileReader = new FileReader();

    fileReader.onloadend = async () =>
    {
        const { vertices, indices } = await ParseObj(new Uint8Array(fileReader.result));
        WriteObj(vertices, indices);
    };

    fileReader.readAsArrayBuffer(files[0]);
}
