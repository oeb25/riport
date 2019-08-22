# Header 1

Det _virker_ **ikke** så godt! :(

## Header 2

Øv det er noget fis... $y = \sum x^2$. ;)

$$
m_f = \int_a^b x\; dx
$$

```python
from datetime import datetime

def add(a, b):
    return a + b

def mul(a, b):
    return a * b

def fib(a):
    if a < 2:
        return a
    else:
        return fib(a - 1) + fib(a - 2)

x = mul(add(2, 3), 2)

pre = datetime.now()

print("fib(" + str(x) + ") = " + str(fib(x)))

post = datetime.now()

print("took " + str(post - pre))
```
