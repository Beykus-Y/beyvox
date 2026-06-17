// AudioWorklet processor: принимает PCM f32 LE байты из Rust через postMessage,
// буферизует и выдаёт в output (MediaStreamAudioDestinationNode → LiveKit)

class RustMicProcessor extends AudioWorkletProcessor {
  constructor() {
    super()
    // Кольцевой буфер: 2 секунды при 48000 Гц
    this._buf = new Float32Array(48000 * 2)
    this._write = 0
    this._read = 0
    this._size = this._buf.length

    this.port.onmessage = (e) => {
      if (e.data === 'stop') {
        this._write = 0
        this._read = 0
        return
      }
      // e.data — Uint8Array: сырые f32 LE байты от Rust
      const bytes = e.data instanceof Uint8Array ? e.data : new Uint8Array(e.data)
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
      const count = Math.floor(bytes.byteLength / 4)
      for (let i = 0; i < count; i++) {
        this._buf[this._write] = view.getFloat32(i * 4, true) // little-endian
        this._write = (this._write + 1) % this._size
        // Если переполнен — двигаем read вперёд (дропаем старые сэмплы)
        if (this._write === this._read) {
          this._read = (this._read + 1) % this._size
        }
      }
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0][0]
    if (!out) return true
    for (let i = 0; i < out.length; i++) {
      if (this._read !== this._write) {
        out[i] = this._buf[this._read]
        this._read = (this._read + 1) % this._size
      } else {
        out[i] = 0
      }
    }
    return true
  }
}

registerProcessor('rust-mic-processor', RustMicProcessor)
