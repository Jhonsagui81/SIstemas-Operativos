import time 
import math
import random

while True:
    for _ in range(1000000):
        # Cálculos más complejos
        x = random.random()
        y = math.sin(x) * math.cos(x)
        math.sqrt(y**2 + x**y)
    time.sleep(0.1)