import * as base64 from 'base64-arraybuffer'

const encoder = new TextEncoder()
const decoder = new TextDecoder()

export function stringToBytes(str: string) {
  return encoder.encode(str)
}

export function bytesToString(data: ArrayBuffer) {
  return decoder.decode(data)
}

export function bytesToHex(data: Uint8Array) {
  return [...data].map(b => b.toString(16).padStart(2, '0')).join('')
}

export function encodeBase64Url(data: Uint8Array) {
  let b64 = base64.encode(data)
  return b64.replace('+', '-').replace('/', '_').replace('=', '')
}

export function decodeBase64Url(b64: string) {
  b64 = b64.replace('-', '+').replace('_', '/')
  b64 = b64.padEnd(b64.length + (4 - b64.length % 4) % 4, '=')
  return base64.decode(b64)
}

// Accounts for offset: https://stackoverflow.com/a/54646864
export function bytesToBuffer(array: Uint8Array) {
  return array.buffer.slice(array.byteOffset, array.byteLength + array.byteOffset)
}

export function rawStringToBytes(str: string): Uint8Array {
  return Uint8Array.from(str, c => c.charCodeAt(0))
}

export function decodeB64(data: string) {
  return new Uint8Array(base64.decode(data))
}
