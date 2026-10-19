"""Background music for a release's promo film (see .claude/skills/promo-video).

    python3 site/scripts/promo-music.py <version> <seconds>

Writes output/promo-<version>/music.wav: a soft piano arpeggio over a held chord, C-G-Am-F at
80 bpm, with a gentle fade in and out and no sound effects. It is synthesised here from sine
waves, so it carries no third-party licence. Make it as long as film.html's window.DURATION.
"""
import math
import struct
import sys
import wave
from pathlib import Path

if len(sys.argv) != 3:
    sys.exit(__doc__)
version, seconds = sys.argv[1], float(sys.argv[2])
out = Path(__file__).resolve().parents[2] / "output" / f"promo-{version}" / "music.wav"
out.parent.mkdir(parents=True, exist_ok=True)

SR = 48000
N = int(SR * seconds)
L, R = [0.0] * N, [0.0] * N


def add(t0, samples, gain, pan=0.0):
    i0 = int(t0 * SR)
    gl, gr = gain * (1 - max(0, pan)), gain * (1 + min(0, pan))
    for k, v in enumerate(samples):
        if 0 <= i0 + k < N:
            L[i0 + k] += v * gl
            R[i0 + k] += v * gr


def piano(freq, dur, decay):
    """A plucked tone: fundamental plus a short-lived octave, exponential decay."""
    n = int(dur * SR)
    return [
        min(1, k / 300)
        * math.exp(-k / (decay * SR))
        * (
            math.sin(2 * math.pi * freq * k / SR)
            + 0.25 * math.sin(4 * math.pi * freq * k / SR) * math.exp(-k / (0.25 * SR))
        )
        for k in range(n)
    ]


def hz(midi):
    return 440 * 2 ** ((midi - 69) / 12)


BAR = 4 * 60 / 80
# (held chord, arpeggio notes) per bar: C, G, Am, F.
CHORDS = [
    ([48, 55, 64], [60, 64, 67, 72]),
    ([43, 55, 62], [59, 62, 67, 71]),
    ([45, 57, 64], [60, 64, 69, 72]),
    ([41, 53, 60], [60, 65, 69, 72]),
]
for bar in range(int(seconds / BAR) + 1):
    t = bar * BAR
    held, arpeggio = CHORDS[bar % 4]
    for note in held:
        add(t, piano(hz(note), BAR + 1.0, 2.2), 0.07)
    for i in range(8):
        note = arpeggio[(0, 1, 2, 3, 2, 1, 2, 3)[i]]
        add(t + i * BAR / 8, piano(hz(note), 1.4, 0.7), 0.05, pan=(-0.25, 0.25)[i % 2])

for k in range(N):
    t = k / SR
    fade = min(1, t / 1.5) * min(1, (seconds - t) / 2.5)
    L[k] *= fade
    R[k] *= fade
peak = max(max(map(abs, L)), max(map(abs, R))) or 1.0
with wave.open(str(out), "wb") as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes(
        b"".join(
            struct.pack("<hh", int(l / peak * 0.7 * 32767), int(r / peak * 0.7 * 32767))
            for l, r in zip(L, R)
        )
    )
print(out)
