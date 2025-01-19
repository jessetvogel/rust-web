'use strict'

let wasmModule = {}

const encoder = new TextEncoder()
const decoder = new TextDecoder()

const objects = []
const free = []

function storeObject(object) {
    const id = free.pop()
    if (id !== undefined) {
        objects[id] = object
        return id
    }
    objects.push(object)
    return objects.length - 1
}

function serializeF64(f) {
    const buffer = new ArrayBuffer(8)
    const view = new DataView(buffer)
    view.setFloat64(0, f, true)
    return new Uint8Array(buffer)
}

function serializeBigInt(i) {
    const buffer = new ArrayBuffer(8)
    const view = new DataView(buffer)
    view.setBigInt64(0, i, true)
    return new Uint8Array(buffer)
}

function serializeU32(i) {
    const buffer = new ArrayBuffer(4)
    const view = new DataView(buffer)
    view.setUint32(0, i, true)
    return new Uint8Array(buffer)
}

function serialize(values) {
    const buffer = []

    const length = values.length
    buffer.push(...new Uint8Array(new Uint32Array([length]).buffer))

    for (const value of values) {
        if (value === undefined) {
            buffer.push(0x00)
        } else if (value === null) {
            buffer.push(0x01)
        } else if (typeof value === 'boolean') {
            buffer.push(value ? 0x02 : 0x03)
        } else if (typeof value === 'number') {
            buffer.push(0x04)
            buffer.push(...serializeF64(value))
        } else if (typeof value === 'bigint') {
            buffer.push(0x05)
            buffer.push(...serializeBigInt(value))
        } else if (typeof value === 'string') {
            buffer.push(0x06)
            const encoded = encoder.encode(value)
            buffer.push(...serializeU32(encoded.length))
            buffer.push(...encoded)
        } else if (value instanceof Uint8Array) {
            buffer.push(0x09)
            buffer.push(...serializeU32(value.length))
            buffer.push(...value)
        } else if (typeof value === 'object') {
            buffer.push(Array.isArray(value) ? 0x07 : 0x08)
            buffer.push(...serializeU32(storeObject(value)))
        } else {
            throw new Error(`could not serialize object of type ${typeof value}`)
        }
    }

    return new Uint8Array(buffer)
}

function deserialize(buffer) {
    const view = new DataView(buffer.buffer)
    const values = []
    let i = 4 // first 4 bytes encode number of values
    while (i < buffer.length) {
        let x = buffer[i]
        if (x == 0x00) {
            values.push(undefined)
            i += 1
        } else if (x == 0x01) {
            values.push(null)
            i += 1
        } else if (x == 0x02) {
            values.push(true)
            i += 1
        } else if (x == 0x03) {
            values.push(false)
            i += 1
        } else if (x == 0x04) {
            values.push(view.getFloat64(i + 1, true))
            i += 1 + 8
        } else if (x == 0x05) {
            values.push(view.getBigInt64(i + 1, true))
            i += 1 + 8
        } else if (x == 0x06) {
            const len = view.getUint32(i + 1, true)
            values.push(decoder.decode(buffer.subarray(i + 1 + 4, i + 1 + 4 + len)))
            i += 1 + 4 + len
        } else if (x == 0x07 || x == 0x08) {
            const id = view.getUint32(i + 1, true)
            values.push(objects[id])
            i += 1 + 4
        } else if (x == 0x09) {
            const len = view.getUint32(i + 1, true)
            values.push(buffer.subarray(i + 1 + 4, i + 1 + 4 + len))
            i += 1 + 4 + len
        } else {
            throw new Error(`invalid parameter type (0x${x.toString(16)})`)
        }
    }
    return values
}

function getWasmImports() {
    const env = {
        __invoke(c_ptr, c_len, p_ptr, p_len) {
            const funcBody = decoder.decode(readBufferFromMemory(c_ptr, c_len));
            const func = Function(`'use strict';return(${funcBody})`)()
            const values = deserialize(readBufferFromMemory(p_ptr, p_len))
            const result = func.call({}, ...values)
            writeBufferToMemory(serialize([result]))
        },
        __free_object(id) {
            objects[id] = undefined
            free.push(id)
        },
        __query_selector(q_ptr, q_len) {
            const query = decoder.decode(readBufferFromMemory(q_ptr, q_len));
            const result = document.querySelector(query);
            writeBufferToMemory(serialize([result]))
        }
    }
    return { env }
}

function readBufferFromMemory(ptr, len) {
    const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
    return new Uint8Array(memory.subarray(ptr, ptr + len))
}

function writeBufferToMemory(buffer) {
    const ptr = wasmModule.instance.exports.get_allocation(buffer.length)
    const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
    memory.set(buffer, ptr)
}

async function init() {
    const imports = getWasmImports()
    const wasmScript = document.querySelector('script[type="application/wasm"]')
    const wasmBuffer = await fetch(wasmScript.src).then(r => r.arrayBuffer())
    wasmModule = await WebAssembly.instantiate(wasmBuffer, imports)
    wasmModule.instance.exports.main()
}

document.addEventListener('DOMContentLoaded', init)
