import numpy as np
import scipy as sp
import scipy.integrate
import scipy.io.wavfile
import matplotlib.pyplot as plt
from pydub import AudioSegment

k = -0.5
n = 4
delay = np.array([
    [0, 0.01, 0.01, 0.03],
    [0.01, 0, 0.02, 0.01],
    [0.01, 0.02, 0, 0.01],
    [0.03, 0.01, 0.01, 0],
])
omega = np.array([
    1, 0.99, 1.01, 1
]) * 2 * np.pi * 2  # BPM = 120

theta_seq = np.zeros((0, n))
init_theta = np.array([0, 2, 1, 3])
theta_seq = np.append(theta_seq, [init_theta], axis=0)
sigma = 0.5

t = 0
eps = 0.001
sec = 30
for i in range(int(sec / eps)):
    t += eps
    if i <= 100:
        dtheta = omega + sigma * np.random.normal(size=n)
    else:
        dtheta = omega + sigma * np.random.normal(size=n)
        for agent_idx1 in range(n):
            for agent_idx2 in range(n):
                if agent_idx1 != agent_idx2:
                    delay_step = int(delay[agent_idx1, agent_idx2] / eps)
                    dtheta[agent_idx1] += (
                        k / n * np.sin(
                            theta_seq[-1, agent_idx1]
                            - theta_seq[-1 - delay_step, agent_idx2]
                        )
                    )
    dtheta = np.maximum(0, dtheta)
    theta_seq = np.append(theta_seq, [theta_seq[-1] + eps * dtheta], axis=0)

plt.figure(figsize=(20, 4))
x = np.sin(theta_seq)
for i in range(n):
    plt.plot(x[:, i])
plt.savefig("tmp.png")

beat = []
for i in range(n):
    x = theta_seq[:, i]
    y = x % (2 * np.pi)
    y = (y[1:] - y[:-1]) < 0
    y = np.where(y)[0]
    a = (np.arange(len(y)) + 1) * 2 * np.pi - x[y]
    b = x[y + 1] - (np.arange(len(y)) + 1) * 2 * np.pi
    beat.append((y + a / (a + b)) * eps)

clap_data = AudioSegment.from_file("clap.mp3", "mp3")
clap_data = np.array(clap_data.get_array_of_samples())
clap_data = clap_data.astype(float)
clap_data /= 2 ** 15

rate = 44100
output_wave = np.zeros(sec * rate)
for i in range(n):
    beat_ = beat[i]
    for t in beat_:
        t = int(t * rate)
        add_size = min(len(output_wave) - t, len(clap_data))
        output_wave[t: t + add_size] += clap_data[:add_size]
output_wave /= np.max(np.abs(output_wave))

plt.clf()
plt.plot(output_wave)
plt.savefig("tmp2.png")

scipy.io.wavfile.write("tmp.wav", rate, output_wave)
