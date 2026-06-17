type OscType = OscillatorType

const ctx = { instance: null as AudioContext | null }

function getCtx(): AudioContext {
  if (!ctx.instance) {
    ctx.instance = new AudioContext()
  }
  if (ctx.instance.state === 'suspended') {
    ctx.instance.resume()
  }
  return ctx.instance
}

function playTone(
  freq: number,
  startDelay: number,
  duration: number,
  startVol: number,
  endVol = 0.0001,
  type: OscType = 'sine',
) {
  const c = getCtx()
  const now = c.currentTime + startDelay
  const osc = c.createOscillator()
  const gain = c.createGain()

  osc.type = type
  osc.connect(gain)
  gain.connect(c.destination)

  osc.frequency.setValueAtTime(freq, now)
  gain.gain.setValueAtTime(startVol, now)
  gain.gain.exponentialRampToValueAtTime(endVol, now + duration)

  osc.start(now)
  osc.stop(now + duration)
}

export const SoundEffects = {
  join() {
    playTone(523.25, 0, 0.15, 0.08)
    playTone(783.99, 0.08, 0.25, 0.08)
  },

  leave() {
    playTone(783.99, 0, 0.15, 0.08)
    playTone(523.25, 0.08, 0.25, 0.08)
  },

  hover() {
    const c = getCtx()
    const now = c.currentTime
    const osc = c.createOscillator()
    const gain = c.createGain()
    osc.connect(gain)
    gain.connect(c.destination)
    osc.frequency.setValueAtTime(800, now)
    osc.frequency.exponentialRampToValueAtTime(150, now + 0.03)
    gain.gain.setValueAtTime(0.015, now)
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.03)
    osc.start(now)
    osc.stop(now + 0.03)
  },

  click() {
    const c = getCtx()
    const now = c.currentTime
    const osc = c.createOscillator()
    const gain = c.createGain()
    osc.connect(gain)
    gain.connect(c.destination)
    osc.frequency.setValueAtTime(1200, now)
    osc.frequency.exponentialRampToValueAtTime(100, now + 0.06)
    gain.gain.setValueAtTime(0.06, now)
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.06)
    osc.start(now)
    osc.stop(now + 0.06)
  },

  muteMic() {
    playTone(349.23, 0, 0.08, 0.07)
    playTone(293.66, 0.06, 0.18, 0.07)
  },

  unmuteMic() {
    playTone(293.66, 0, 0.08, 0.07)
    playTone(349.23, 0.06, 0.18, 0.07)
  },

  deafen() {
    playTone(329.63, 0, 0.12, 0.07)
    playTone(261.63, 0.08, 0.15, 0.07)
    playTone(196.00, 0.16, 0.25, 0.07)
  },

  undeafen() {
    playTone(196.00, 0, 0.12, 0.07)
    playTone(261.63, 0.08, 0.15, 0.07)
    playTone(329.63, 0.16, 0.25, 0.07)
  },
}

function playNote(freq: number, startTime: number, duration: number, vol: number) {
  const c = getCtx()
  const osc = c.createOscillator()
  const gain = c.createGain()
  osc.type = 'sine'
  osc.connect(gain)
  gain.connect(c.destination)
  osc.frequency.setValueAtTime(freq, startTime)
  gain.gain.setValueAtTime(vol, startTime)
  gain.gain.exponentialRampToValueAtTime(0.0001, startTime + duration)
  osc.start(startTime)
  osc.stop(startTime + duration)
}

let callInterval: ReturnType<typeof setInterval> | null = null

export const PhoneCallSounds = {
  _incomingCycle() {
    const c = getCtx()
    const now = c.currentTime
    const vol = 0.06
    const len = 0.15
    playNote(1046.50, now + 0.0, len, vol)
    playNote(1318.51, now + 0.1, len, vol)
    playNote(1046.50, now + 0.2, len, vol)
    playNote(1567.98, now + 0.3, len, vol)
    playNote(1046.50, now + 0.6, len, vol)
    playNote(1318.51, now + 0.7, len, vol)
    playNote(1046.50, now + 0.8, len, vol)
    playNote(1567.98, now + 0.9, len, vol)
  },

  _outgoingBeep() {
    const c = getCtx()
    const now = c.currentTime
    const osc1 = c.createOscillator()
    const osc2 = c.createOscillator()
    const gain = c.createGain()
    osc1.connect(gain)
    osc2.connect(gain)
    gain.connect(c.destination)
    osc1.frequency.setValueAtTime(350, now)
    osc2.frequency.setValueAtTime(440, now)
    gain.gain.setValueAtTime(0.0, now)
    gain.gain.linearRampToValueAtTime(0.04, now + 0.2)
    gain.gain.setValueAtTime(0.04, now + 1.0)
    gain.gain.exponentialRampToValueAtTime(0.0001, now + 1.4)
    osc1.start(now); osc2.start(now)
    osc1.stop(now + 1.4); osc2.stop(now + 1.4)
  },

  startIncoming() {
    this.stop()
    this._incomingCycle()
    callInterval = setInterval(() => this._incomingCycle(), 2500)
  },

  startOutgoing() {
    this.stop()
    this._outgoingBeep()
    callInterval = setInterval(() => this._outgoingBeep(), 4000)
  },

  stop() {
    if (callInterval !== null) {
      clearInterval(callInterval)
      callInterval = null
    }
  },
}
