import plotly.express as px
import pandas as pd

# Nanoseconds in a second
NANOS_IN_SEC = 1000000000

# Plotting step in nanos
STEP = 100


# Like https://doc.rust-lang.org/std/primitive.u32.html#method.div_ceil
def div_ceil(a, b):
    return -(a // -b)


# Like https://doc.rust-lang.org/std/primitive.u32.html#method.div_floor
def div_floor(a, b):
    return a // b


file = open("foo.txt", "r")
parts = [line.split() for line in file.readlines()]
print(parts)

# Update times to be in nanos and get max and min times
min_time = pow(2, 31)
max_time = 0
for part in parts:
    secs, nanos = part[0].split(":")
    part[0] = int(secs) * NANOS_IN_SEC + int(nanos)
    if part[0] > max_time:
        max_time = part[0]
    if part[0] < min_time:
        min_time = part[0]
min_time = STEP * div_floor(min_time, STEP)
max_time = STEP * div_ceil(max_time, STEP)
# print("min_time:", min_time)
# print("max_time:", max_time)

# Combined allocations and deallocations into single elements
combined_parts = {}
for part in parts:
    if part[1] == "+":
        addr = part[2]
        combined_parts[addr] = [int(part[3]), part[0], max_time]
    else:
        addr = part[2]
        assert int(part[3]) == combined_parts[addr][0]
        combined_parts[addr][2] = part[0]
print(combined_parts)

# Create samples at given step
extended_parts = {}
for addr, value in combined_parts.items():
    size, start, stop = value
    new = []
    for step in range(min_time, max_time, STEP):
        if step in range(start, stop):
            new.append(size)
        else:
            new.append(0)
    extended_parts[addr] = new
# print(extended_parts)

# Convert into flat array
flattened_parts = []
for addr, value in extended_parts.items():
    for step, size in zip(range(min_time, max_time, STEP), value):
        flattened_parts.append([addr, step, size])
# print(flattened_parts)

df = pd.DataFrame(flattened_parts, columns=["Address", "Nanos", "Size"])
# df["Address"] = df["Address"].apply(hex)
print(df)

fig = px.area(df, x="Nanos", y="Size", color="Address")
fig.show()
