import numpy as np
import scipy as sp
import scipy.integrate
import matplotlib.pyplot as plt

k = -0.1
n = 4
delay = np.array([
    [0, 0.01, 0.01, 0.03],
    [0.01, 0, 0.02, 0.01],
    [0.01, 0.02, 0, 0.01],
    [0.03, 0.01, 0.01, 0],
])
omega = np.array([
    1, 0.99, 1.01, 1
])

theta_seq = np.zeros((0, n))
init_theta = np.array([0, 2, 1, 3])
theta_seq = np.append(theta_seq, [init_theta], axis=0)
sigma = 0.5

t = 0
eps = 0.01
for i in range(20000):
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
    theta_seq = np.append(theta_seq, [theta_seq[-1] + eps * dtheta], axis=0)

plt.figure(figsize=(20, 4))
x = np.sin(theta_seq)
for i in range(n):
    plt.plot(x[:, i])
plt.savefig("tmp.png")
