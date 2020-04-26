import numpy as np
import scipy as sp
import scipy.integrate
import matplotlib.pyplot as plt


m = 1
k = 1
gamma = 0.2
def f(time, state):
    x = state[0]
    v = state[1]
    a = (- k * x - gamma * v + p(x, v)) / m
    return np.array([v, a])

def p(x, v):
    if v > 0:
        return p_plus(x)
    else:
        return p_minus(x)

x1 = 0.01
x2 = 0.11
p1 = 0.4
def p_plus(x):
    p = 0
    if x1 < x and x < x2:
        if x < (x1 + x2) / 2:
            p += (x - x1) / ((x2 - x1) / 2) * p1
        else:
            p += (x2 - x) / ((x2 - x1) / 2) * p1
    return p

def p_minus(x):
    return -p_plus(-x)


state0 = np.array([0.02, 0])

sol = scipy.integrate.solve_ivp(f, [0, 100], state0,  t_eval=np.linspace(0, 100, 1001))

plt.plot(sol.y[0], sol.y[1])
plt.savefig("tmp.png")
