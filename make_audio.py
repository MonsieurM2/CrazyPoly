"""Generate the CrazyPoly soundtrack and sound effects.

Everything here is synthesised from scratch (sines, noise, a small Schroeder
reverb), so the audio shipped with the game is original and free of any
third-party rights. Run `python make_audio.py` to rebuild assets/audio/*.wav.
"""
import math
import os
import random
import struct
import wave

SR = 44100
OUT = os.path.join("assets", "audio")


def write(name, samples, peak=0.72):
    top = max(1e-9, max(abs(s) for s in samples))
    gain = peak / top
    data = b"".join(
        struct.pack("<h", int(max(-1.0, min(1.0, s * gain)) * 32000)) for s in samples
    )
    with wave.open(os.path.join(OUT, name), "wb") as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(SR)
        f.writeframes(data)
    print("%-12s %5.2fs" % (name, len(samples) / SR))


# ------------------------------------------------------------------ helpers


def adsr(t, attack, decay, sustain=0.0, release=0.25):
    if t < attack:
        return t / attack
    t -= attack
    if sustain <= 0.0:
        return math.exp(-t / decay)
    if t < decay:
        return 1.0 - (1.0 - sustain) * (t / decay)
    return sustain * math.exp(-(t - decay) / (decay * 0.6))


def tone(buf, start, dur, freq, amp, partials=((1.0, 1.0),), attack=0.01, decay=0.4):
    i0 = int(start * SR)
    n = int(dur * SR)
    for k in range(n):
        idx = i0 + k
        if idx >= len(buf):
            break
        t = k / SR
        e = adsr(t, attack, decay)
        v = 0.0
        for mult, a in partials:
            v += a * math.sin(2 * math.pi * freq * mult * t)
        buf[idx] += v * e * amp


def noise_burst(buf, start, dur, amp, low=0.0, high=1.0, decay=0.05):
    """Filtered noise: `low`/`high` are one-pole coefficients (0..1)."""
    i0 = int(start * SR)
    n = int(dur * SR)
    lp = 0.0
    for k in range(n):
        idx = i0 + k
        if idx >= len(buf):
            break
        x = random.uniform(-1.0, 1.0)
        lp += (x - lp) * high if high < 1.0 else x
        v = lp
        if low > 0.0:
            v = lp - low  # biased high-pass substitute, cheap but fine for air
        buf[idx] += v * amp * math.exp(-k / (decay * SR))



def swept(buf, start, dur, f0, f1, amp, attack=0.006, decay=0.06,
          partials=((1.0, 1.0),)):
    """A gliding tone with a rounded attack: no hard edge, still an impact."""
    i0 = int(start * SR)
    n = int(dur * SR)
    ph = 0.0
    for k in range(n):
        idx = i0 + k
        if idx >= len(buf):
            break
        t = k / SR
        f = f0 * (f1 / f0) ** min(1.0, t / dur)
        ph += 2 * math.pi * f / SR
        a = 0.5 - 0.5 * math.cos(math.pi * min(1.0, t / attack)) if attack > 0 else 1.0
        e = a * math.exp(-t / decay)
        v = 0.0
        for mult, gain in partials:
            v += gain * math.sin(ph * mult)
        buf[idx] += v * e * amp


def noise_soft(buf, start, dur, amp, lowpass=0.1, attack=0.008, decay=0.05):
    """Low-passed noise that swells in and dies away: felt, not a click."""
    i0 = int(start * SR)
    n = int(dur * SR)
    lp = 0.0
    for k in range(n):
        idx = i0 + k
        if idx >= len(buf):
            break
        t = k / SR
        lp += (random.uniform(-1.0, 1.0) - lp) * lowpass
        a = 0.5 - 0.5 * math.cos(math.pi * min(1.0, t / attack)) if attack > 0 else 1.0
        buf[idx] += lp * amp * a * math.exp(-t / decay)


def reverb(buf, wet=0.28):
    combs = [(1557, 0.79), (1617, 0.77), (1491, 0.74), (1277, 0.72)]
    acc = [0.0] * (len(buf) + 20000)
    for delay, fb in combs:
        line = [0.0] * delay
        pos = 0
        for i, x in enumerate(buf):
            y = line[pos]
            line[pos] = x + y * fb
            pos = (pos + 1) % delay
            acc[i] += y * 0.25
    # one allpass to smear it
    delay = 225
    line = [0.0] * delay
    pos = 0
    for i in range(len(buf)):
        y = line[pos]
        line[pos] = acc[i] + y * 0.5
        pos = (pos + 1) % delay
        acc[i] = acc[i] + (-y * 0.5)
    for i in range(len(buf)):
        buf[i] = buf[i] * (1.0 - wet) + acc[i] * wet


def midi(n):
    return 440.0 * 2 ** ((n - 69) / 12.0)


# -------------------------------------------------------------------- music

def music():
    bpm = 72.0
    beat = 60.0 / bpm
    bar = 4 * beat
    bars = 8
    tail = 2.0
    length = int((bars * bar + tail) * SR)
    buf = [0.0] * length

    # Fmaj7 - Dm7 - Gm7 - C7, two bars each.
    chords = [
        [53, 57, 60, 64],
        [50, 53, 57, 60],
        [55, 58, 62, 65],
        [48, 52, 55, 58],
    ]
    rng = random.Random(7)

    for b in range(bars):
        chord = chords[(b // 2) % 4]
        t0 = b * bar

        # warm pad: slow, quiet, with a gentle vibrato-free stack
        for note in chord:
            tone(
                buf,
                t0,
                bar * 1.15,
                midi(note),
                0.055,
                partials=((1.0, 1.0), (2.0, 0.28), (3.0, 0.10)),
                attack=0.55,
                decay=2.4,
            )

        # bass: root on 1 and 3, fifth on the "and" of 2
        root = chord[0] - 12
        tone(buf, t0, 1.2, midi(root), 0.30, partials=((1.0, 1.0), (2.0, 0.35)), attack=0.01, decay=0.55)
        tone(buf, t0 + 2 * beat, 1.2, midi(root), 0.26, partials=((1.0, 1.0), (2.0, 0.30)), attack=0.01, decay=0.50)
        tone(buf, t0 + 3.5 * beat, 0.6, midi(root + 7), 0.18, partials=((1.0, 1.0),), attack=0.01, decay=0.35)

        # comping: electric-piano plucks, lazy swing
        pattern = [(0.0, 0), (1.5, 1), (2.5, 2), (3.0, 3), (3.75, 1)]
        for k, (pos, idx) in enumerate(pattern):
            note = chord[idx % len(chord)] + (12 if rng.random() < 0.25 else 0)
            amp = 0.11 * (0.85 + 0.3 * rng.random())
            tone(
                buf,
                t0 + pos * beat,
                1.4,
                midi(note),
                amp,
                partials=((1.0, 1.0), (2.0, 0.45), (4.0, 0.12), (7.0, 0.05)),
                attack=0.004,
                decay=0.55,
            )

        # brushed ride: quiet air on every beat
        for s in range(8):
            amp = 0.022 if s % 2 else 0.030
            noise_burst(buf, t0 + s * beat * 0.5, 0.18, amp, high=0.55, decay=0.03)

    reverb(buf, 0.30)

    # Fold the tail back over the head so the loop is seamless.
    n = int(bars * bar * SR)
    out = [0.0] * n
    for i in range(n):
        v = buf[i]
        if i + n < length:
            v += buf[i + n] * 0.85
        out[i] = v
    # 30 ms fades at the seams to avoid any click
    fade = int(0.03 * SR)
    for i in range(fade):
        k = i / fade
        out[i] *= k
        out[n - 1 - i] *= k
    return out


# ---------------------------------------------------------------- sfx set

def sfx_click():
    buf = [0.0] * int(0.07 * SR)
    noise_burst(buf, 0.0, 0.05, 0.35, high=0.8, decay=0.012)
    tone(buf, 0.0, 0.05, 1450.0, 0.30, partials=((1.0, 1.0),), attack=0.001, decay=0.02)
    return buf


def sfx_dice():
    buf = [0.0] * int(0.95 * SR)
    rng = random.Random(3)
    t = 0.0
    while t < 0.62:
        amp = 0.5 * (1.0 - t / 0.9) + 0.12
        noise_burst(buf, t, 0.05, amp, high=0.75, decay=0.012)
        tone(buf, t, 0.03, rng.uniform(320.0, 900.0), amp * 0.35, attack=0.001, decay=0.012)
        t += rng.uniform(0.018, 0.075)
    # the last knock, as the cubes settle
    tone(buf, 0.66, 0.28, 190.0, 0.55, partials=((1.0, 1.0), (2.0, 0.3)), attack=0.002, decay=0.09)
    noise_burst(buf, 0.66, 0.12, 0.30, high=0.6, decay=0.02)
    return buf


def sfx_step(seed, pitch=210.0):
    """A piece set down on felt: a soft body that settles, a wooden tock and a
    breath of felt, all rounded at the attack so the step never clicks."""
    state = random.getstate()
    random.seed(seed)
    buf = [0.0] * int(0.30 * SR)
    # The weight of the piece landing, gliding down as the felt takes it.
    swept(buf, 0.0, 0.26, pitch, pitch * 0.60, 0.60,
          attack=0.007, decay=0.075, partials=((1.0, 1.0), (2.0, 0.16)))
    # A small wooden tock, an octave and a half up, very round.
    swept(buf, 0.006, 0.17, pitch * 2.7, pitch * 2.25, 0.15,
          attack=0.011, decay=0.038, partials=((1.0, 1.0), (3.0, 0.10)))
    # The felt itself: a quiet swell of noise, gone before it is heard.
    noise_soft(buf, 0.0, 0.20, 0.26, lowpass=0.09, attack=0.009, decay=0.048)
    reverb(buf, 0.13)
    random.setstate(state)
    return buf


def sfx_cash():
    buf = [0.0] * int(0.8 * SR)
    for k, (start, freq) in enumerate([(0.0, 1318.5), (0.10, 1760.0), (0.20, 2093.0)]):
        tone(buf, start, 0.55, freq, 0.30 - k * 0.05, partials=((1.0, 1.0), (2.0, 0.3)), attack=0.002, decay=0.16)
    reverb(buf, 0.22)
    return buf


def sfx_buy():
    buf = [0.0] * int(0.9 * SR)
    notes = [60, 64, 67, 72]
    for k, n in enumerate(notes):
        tone(buf, k * 0.085, 0.7, midi(n), 0.26, partials=((1.0, 1.0), (2.0, 0.35), (3.0, 0.12)), attack=0.004, decay=0.22)
    tone(buf, 0.36, 0.5, midi(48), 0.22, partials=((1.0, 1.0),), attack=0.01, decay=0.25)
    reverb(buf, 0.25)
    return buf


def sfx_card():
    buf = [0.0] * int(0.42 * SR)
    n = len(buf)
    lp = 0.0
    for k in range(n):
        t = k / SR
        x = random.uniform(-1.0, 1.0)
        cutoff = 0.05 + 0.5 * math.sin(math.pi * min(1.0, t / 0.42))
        lp += (x - lp) * cutoff
        buf[k] += lp * 0.4 * math.sin(math.pi * min(1.0, t / 0.42)) ** 2
    return buf


def sfx_jail():
    buf = [0.0] * int(1.0 * SR)
    tone(buf, 0.0, 0.55, 160.0, 0.5, partials=((1.0, 1.0), (2.0, 0.4), (3.0, 0.2)), attack=0.003, decay=0.16)
    noise_burst(buf, 0.0, 0.25, 0.35, high=0.35, decay=0.05)
    # heavy clank of the door
    for start in (0.22, 0.30):
        noise_burst(buf, start, 0.2, 0.25, high=0.9, decay=0.03)
        tone(buf, start, 0.2, 740.0, 0.14, partials=((1.0, 1.0), (2.7, 0.5)), attack=0.001, decay=0.05)
    reverb(buf, 0.25)
    return buf


def sfx_build():
    buf = [0.0] * int(0.55 * SR)
    for start in (0.0, 0.16):
        tone(buf, start, 0.2, 210.0, 0.42, partials=((1.0, 1.0), (2.0, 0.35)), attack=0.001, decay=0.05)
        noise_burst(buf, start, 0.08, 0.30, high=0.85, decay=0.015)
    return buf


def sfx_bad():
    buf = [0.0] * int(0.45 * SR)
    for k in range(int(0.32 * SR)):
        t = k / SR
        e = math.exp(-t / 0.12)
        phase = 2 * math.pi * (196.0 - 40.0 * t) * t
        v = math.copysign(1.0, math.sin(phase)) * 0.5 + math.sin(phase) * 0.5
        buf[k] += v * e * 0.35
    return buf


def sfx_win():
    buf = [0.0] * int(1.8 * SR)
    notes = [60, 64, 67, 72, 76, 79, 84]
    for k, n in enumerate(notes):
        tone(buf, k * 0.11, 1.4, midi(n), 0.26, partials=((1.0, 1.0), (2.0, 0.4), (3.0, 0.2)), attack=0.003, decay=0.35)
    for n in (48, 55, 60, 64):
        tone(buf, 0.75, 1.0, midi(n), 0.16, partials=((1.0, 1.0), (2.0, 0.3)), attack=0.02, decay=0.5)
    reverb(buf, 0.3)
    return buf


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    random.seed(11)
    write("music.wav", music(), peak=0.62)
    write("click.wav", sfx_click(), peak=0.55)
    write("dice.wav", sfx_dice(), peak=0.70)
    write("step.wav", sfx_step(21, 210.0), peak=0.42)
    write("step2.wav", sfx_step(47, 196.0), peak=0.42)
    write("step3.wav", sfx_step(83, 226.0), peak=0.42)
    write("cash.wav", sfx_cash(), peak=0.60)
    write("buy.wav", sfx_buy(), peak=0.62)
    write("card.wav", sfx_card(), peak=0.50)
    write("jail.wav", sfx_jail(), peak=0.65)
    write("build.wav", sfx_build(), peak=0.60)
    write("bad.wav", sfx_bad(), peak=0.55)
    write("win.wav", sfx_win(), peak=0.70)
