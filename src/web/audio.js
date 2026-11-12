export class Audio {
  enabled = false;
  context;
  last = new Map();

  async toggle() {
    this.context ??= new AudioContext();
    await this.context.resume();
    this.enabled = !this.enabled;
    if (this.enabled) this.play("upgrade");
    return this.enabled;
  }

  play(cue) {
    if (!this.enabled || !this.context || this.context.state !== "running")
      return;
    const now = this.context.currentTime;
    if (now - (this.last.get(cue) ?? -1) < (cue === "shoot" ? 0.1 : 0.07))
      return;
    this.last.set(cue, now);
    const sounds = {
      shoot: [520, 160, 0.045, 0.018, "square"],
      pickup: [880, 1320, 0.045, 0.022, "sine"],
      hit: [110, 35, 0.14, 0.07, "sawtooth"],
      dash: [180, 700, 0.12, 0.035, "triangle"],
      ability: [80, 420, 0.35, 0.07, "sawtooth"],
      level: [440, 1760, 0.25, 0.04, "square"],
      upgrade: [660, 1320, 0.17, 0.035, "triangle"],
      event: [330, 660, 0.3, 0.04, "triangle"],
      boss: [220, 55, 0.8, 0.06, "sawtooth"],
      lose: [330, 40, 0.7, 0.06, "triangle"],
      win: [440, 1760, 0.8, 0.05, "triangle"],
    };
    const sound = sounds[cue];
    if (!sound) return;
    const [from, to, duration, volume, type] = sound;
    const oscillator = this.context.createOscillator();
    const gain = this.context.createGain();
    oscillator.type = type;
    oscillator.frequency.setValueAtTime(from, now);
    oscillator.frequency.exponentialRampToValueAtTime(to, now + duration);
    gain.gain.setValueAtTime(0, now);
    gain.gain.linearRampToValueAtTime(volume, now + 0.006);
    gain.gain.exponentialRampToValueAtTime(0.001, now + duration);
    oscillator.connect(gain).connect(this.context.destination);
    oscillator.start(now);
    oscillator.stop(now + duration + 0.01);
    oscillator.onended = () => {
      oscillator.disconnect();
      gain.disconnect();
    };
  }
}
