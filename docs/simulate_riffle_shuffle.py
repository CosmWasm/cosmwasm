import functools

# Create a funtion that executed f recusively n times, i.e. f**n
def power(f, n):
    functions = [f for _ in range(n)]
    def compose2(f, g):
        return lambda x: f(g(x))
    return functools.reduce(compose2, functions, lambda x: x)

# Rotate input to the left by n positions
def rotate_left(input, n):
    return input[n:] + input[0:n]

def riffle_shuffle(input):
    left = input[0:len(input)//2]
    right = input[len(input)//2:]
    i = 0
    out = ""
    while i < len(input)//2:
        out += right[i] + left[i]
        i += 1
    return out

values = [
    "alice123----------------", # 0
    "alice485----------------", # 1
    "aliceimwunderland521----", # 2
    "bob1--------------------", # 3
    "bob123------------------", # 4
    "bob485------------------", # 5
    "bob511------------------", # 6
    "creator-----------------", # 7
]

def digit_sum(input):
    def value(char):
        if char == "-":
            return 0
        else:
            return ord(char)
    return sum([value(c) for c in input])

shuffle = power(riffle_shuffle, 18)
rotated = [rotate_left(v, digit_sum(v) % 24) for v in values]
rotated_shuffled = [shuffle(r) for r in rotated]
shuffled = [shuffle(v) for v in values]

print("Original:\n" + "\n".join(sorted(values)))
print()
# digit_sums = [str(digit_sum(v) % 24) for v in values]
# print("Digit sums:\n" + "\n".join(digit_sums))
# print()
print("Rotated:\n" + "\n".join(sorted(rotated)))
print()
print("Shuffled:\n" + "\n".join(sorted(shuffled)))
print()
print("Rotated+Shuffled:\n" + "\n".join(sorted(rotated_shuffled)))
