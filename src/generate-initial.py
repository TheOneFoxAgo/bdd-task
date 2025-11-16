#!/usr/bin/env python3

from random import shuffle

properties = [list(range(9)) for _ in range(4)]
for p in properties:
    shuffle(p)
for i in range(9):
    first = properties[0][i]
    second = properties[1][i]
    third = properties[2][i]
    forth = properties[3][i]
    print(f"Объект {i}: {first}, {second}, {third}, {forth}")
