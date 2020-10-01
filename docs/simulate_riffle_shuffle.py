import functools

# Create a funtion that executed f recusively n times, i.e. f**n
def power(f, n):
    functions = [f for _ in range(n)]
    def compose2(f, g):
        return lambda x: f(g(x))
    return functools.reduce(compose2, functions, lambda x: x)

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

transform = power(riffle_shuffle, 18)
transformed = [transform(v) for v in values]

print("Original:\n" + "\n".join(sorted(values)))
print()
print("Shuffled:\n" + "\n".join(sorted(transformed)))
