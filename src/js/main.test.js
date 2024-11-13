const test = require('node:test')
const assert = require('node:assert')

const { readParamsFromMemory, writeBufferToMemory, wasmModule } = require('./main')

// node src/js/main.test.js

test('check read params', () => {

    const float64View = new DataView(new ArrayBuffer(8))
    float64View.setFloat64(0, 42.42, true)
    const float64Array = new Uint8Array(float64View.buffer)

    const bigInt64View = new DataView(new ArrayBuffer(8))
    bigInt64View.setBigInt64(0, 42n, true)
    const bigInt64Array = new Uint8Array(bigInt64View.buffer)

    const uint32View = new DataView(new ArrayBuffer(4))
    uint32View.setUint32(0, 42, true)
    const uint32Array = new Uint8Array(uint32View.buffer)

    const testCases = [
        {memory: [0], expected: [undefined]},
        {memory: [1], expected: [null]},
        {memory: [2, ...float64Array], expected: [42.42]},
        {memory: [3, ...bigInt64Array], expected: [42n]},
        {memory: [4, ...uint32Array, ...uint32Array], expected: ['']},
        {memory: [5], expected: [true]},
        {memory: [6], expected: [false]},
        {memory: [7, ...uint32Array], expected: [undefined]},
    ]
    for (const testCase of testCases) {
        wasmModule.instance = { exports: { memory: { buffer: testCase.memory } } }

        const result = readParamsFromMemory(0, testCase.memory.length)
        assert.deepStrictEqual(result, testCase.expected)
    }
})

test('check write buffer', () => {

    const testCases = [
        {memory: [], expected: 0},
    ]
    const create_allocation = () => { return 0 }
    const get_allocation = () => { return 0 }
    for (const testCase of testCases) {

        const exports = { create_allocation, get_allocation, memory: { buffer: testCase.memory } }
        wasmModule.instance = { exports }

        const result = writeBufferToMemory(0, [])
        assert.deepStrictEqual(result, testCase.expected)
    }
})
