# sabri
my favorite working multi-paradigm, interpreted programming language.

## draft

basic
```
str := "a string"
str = 123 ~ dynamic

~ empty declaration?
null? :=
```

funcs
```
~ a comment
greet := |a|
  print("yo, " + a)

hello_world = | greet("world")
```

ifs
```
fib := |a|
  if a < 3 then return a
  return fib(a - 1) + fib(a - 2)

if 100 + r'a string' == '100a string'
  print(fib(100))
```

data
```
~ todo
t := {
    a: 'a table member'
    fun: |a, b|
      return a + b
}

print(t.a) ~=> 'a table member
print(3 == t.fun(1, 2)) ~=> true
```
